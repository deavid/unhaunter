//! # Level Finalization Module
//!
//! This module handles post-load processing for levels, including:
//! - Temperature field initialization and smoothing
//! - Processing mesh placeholders into actual mesh instances
//! - Adding prebaked lighting to the level
//! - Computing usable area statistics for the level

use bevy::prelude::*;
use bevy::utils::HashMap;
use rand::Rng;
use uncore::behavior::Behavior;
use uncore::components::board::boardposition::BoardPosition;
use uncore::components::board::position::Position;
use uncore::events::loadlevel::LevelReadyEvent;
use uncore::events::roomchanged::RoomChangedEvent;
use uncore::resources::board_data::BoardData;
use uncore::resources::roomdb::RoomDB;
use uncore::states::AppState;
use uncore::{celsius_to_kelvin, random_seed};
use unlight::prebake::prebake_lighting_field;
use unstd::board::tiledata::PreMesh;
use unstd::plugins::board::rebuild_collision_data;

/// Processes level completion after the level is fully loaded.
///
/// This system:
/// - Transitions to the in-game state
/// - Initializes the temperature field with randomized values
/// - Makes cold spots near the ghost breach
/// - Calculates usable area statistics for each floor
/// - Performs temperature field smoothing for a more natural distribution
///
/// # Arguments
/// * `bf` - Board data resource to modify
/// * `ev` - Event reader for level ready events
/// * `ev_room` - Event writer for room changed events
/// * `roomdb` - Room database resource for room information
/// * `next_game_state` - State machine to transition to in-game state
pub fn after_level_ready(
    mut bf: ResMut<BoardData>,
    mut ev: EventReader<LevelReadyEvent>,
    mut ev_room: EventWriter<RoomChangedEvent>,
    roomdb: Res<RoomDB>,
    mut next_game_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
) {
    if ev.is_empty() {
        return;
    }

    // Get RNG and level parameters
    let mut rng = random_seed::rng();
    let open_van = ev.read().next().unwrap().open_van;

    // Switch to in-game state
    next_game_state.set(AppState::InGame);
    bf.level_ready_time = time.elapsed_secs();

    // Store ambient temperature for reference
    let ambient_temp = bf.ambient_temp;

    // Identify the room containing the ghost breach
    let breach_room = roomdb.room_tiles.get(&bf.breach_pos.to_board_position());

    // Randomize initial temperatures to create a more realistic distribution
    for (idxpos, temperature) in bf.temperature_field.indexed_iter_mut() {
        let room = roomdb.room_tiles.get(&BoardPosition::from_ndidx(idxpos));

        // Make the breach room cold
        if room == breach_room {
            *temperature = celsius_to_kelvin(0.5); // Near freezing
        } else {
            // Other rooms get ambient temp with slight variance
            let ambient = ambient_temp + rng.random_range(-3.0..3.0);
            *temperature = ambient;
        }
    }

    // Send room changed event with van open state
    ev_room.send(RoomChangedEvent::init(open_van));

    // --- Calculate Usable Haunted Area ---
    let mut total_usable_area_m2 = 0.0;
    let mut area_per_floor: HashMap<i64, f32> = HashMap::new();
    let mut rooms_per_floor: HashMap<i64, HashMap<String, f32>> = HashMap::new();
    let tile_area = BoardPosition::area_per_tile_m2();

    // Calculate usable area for each floor and room (excluding solid walls)
    for bpos in roomdb.room_tiles.keys() {
        if let Some(cf) = bf.collision_field.get(bpos.ndidx()) {
            // Exclude static walls (opaque and not dynamic)
            if cf.see_through || cf.is_dynamic {
                // Get the floor and increment its area
                let floor_area = area_per_floor.entry(bpos.z).or_insert(0.0);
                *floor_area += tile_area;

                // Add to room area calculation, organized by floor
                if let Some(room_name) = roomdb.room_tiles.get(bpos) {
                    let floor_rooms = rooms_per_floor.entry(bpos.z).or_insert_with(HashMap::new);
                    let room_area = floor_rooms.entry(room_name.clone()).or_insert(0.0);
                    *room_area += tile_area;
                }

                total_usable_area_m2 += tile_area;
            }
        } else {
            warn!(
                "Tile at {:?} found in RoomDB but not in behavior_field.",
                bpos
            );
        }
    }

    // Log area statistics for each floor and its rooms
    info!("--- Usable Haunted Area Calculation ---");
    let mut sorted_floors: Vec<_> = area_per_floor.keys().collect();
    sorted_floors.sort();

    for floor_z in sorted_floors {
        info!("--- Floor {} ---", floor_z);

        // Print rooms for this floor
        if let Some(floor_rooms) = rooms_per_floor.get(floor_z) {
            let mut sorted_rooms: Vec<_> = floor_rooms.keys().collect();
            sorted_rooms.sort();

            for room_name in sorted_rooms {
                if let Some(area) = floor_rooms.get(room_name) {
                    info!("  Room '{}': {:.2} m²", room_name, area);
                }
            }
        }

        // Print floor total
        if let Some(area) = area_per_floor.get(floor_z) {
            info!("  Floor {} Total: {:.2} m²", floor_z, area);
        }
    }

    info!("Grand Total Usable Area: {:.2} m²", total_usable_area_m2);
    info!("--------------------------------------");

    // Smooth temperature field to avoid abrupt changes
    warn!(
        "Computing 32x{:?} = {}",
        bf.map_size,
        32 * bf.map_size.0 * bf.map_size.1 * bf.map_size.2
    );

    // Apply temperature smoothing iterations
    for _ in 0..32 {
        let temp_snap = bf.temperature_field.clone();
        for z in 0..bf.map_size.2 {
            for y in 0..bf.map_size.1 {
                for x in 0..bf.map_size.0 {
                    let p = (x, y, z);
                    let free_tot =
                        bf.collision_field[p].player_free || bf.collision_field[p].is_dynamic;
                    if !free_tot {
                        continue;
                    }
                    let bpos = BoardPosition::from_ndidx(p);
                    let nbors = bpos.iter_xy_neighbors(1, bf.map_size);
                    let mut t_temp = temp_snap.get(p).copied().unwrap_or(ambient_temp);
                    let mut count = 1.0;
                    if t_temp < celsius_to_kelvin(1.0) {
                        // Don't warm up the cold ghost room during init.
                        continue;
                    }
                    for npos in nbors {
                        let free = bf
                            .collision_field
                            .get(npos.ndidx())
                            .map(|x| x.player_free || x.is_dynamic)
                            .unwrap_or(true);
                        if free {
                            t_temp += temp_snap.get(npos.ndidx()).copied().unwrap_or(ambient_temp);
                            count += 1.0;
                        }
                    }
                    t_temp /= count;
                    bf.temperature_field[p] = t_temp;
                }
            }
        }
    }
    warn!("Done: Computing 16x");
}

/// Processes sprite placeholders (PreMesh) into actual mesh components.
///
/// During level loading, entities are given PreMesh components as placeholders.
/// This system runs after level loading to convert those into actual Mesh components.
///
/// # Arguments
/// * `commands` - Command buffer for entity modifications
/// * `query` - Query to find entities with PreMesh components
/// * `images` - Asset storage for image data
/// * `meshes` - Asset storage for mesh creation
pub fn process_pre_meshes(
    mut commands: Commands,
    query: Query<(Entity, &PreMesh)>,
    images: Res<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, pre_mesh) in query.iter() {
        match pre_mesh {
            // For mesh placeholders, simply apply the existing mesh
            PreMesh::Mesh(mesh2d) => {
                commands
                    .entity(entity)
                    .insert(mesh2d.clone())
                    .remove::<PreMesh>();
            }
            // For image placeholders, create a mesh from the image dimensions
            PreMesh::Image {
                sprite_anchor,
                image_handle,
            } => {
                if let Some(image) = images.get(image_handle) {
                    let sz = image.texture_descriptor.size;
                    println!("Physical image size: {} x {}", sz.width, sz.height);
                    let sprite_size = Vec2::new(sz.width as f32, sz.height as f32);
                    let sprite_anchor = Vec2::new(
                        sprite_size.x * sprite_anchor.x,
                        sprite_size.y * sprite_anchor.y,
                    );

                    // Create quad mesh with proper dimensions and anchor point
                    let base_quad = Mesh::from(uncore::types::quadcc::QuadCC::new(
                        sprite_size,
                        sprite_anchor,
                    ));
                    let mesh_handle = meshes.add(base_quad);
                    let mesh2d = Mesh2d::from(mesh_handle);

                    // Replace PreMesh with actual Mesh
                    commands.entity(entity).insert(mesh2d).remove::<PreMesh>();
                    println!("Processed entity: {:?}", entity);
                }
            }
        }
    }
}

/// Adds prebaked lighting to the level after loading is complete.
///
/// This function:
/// - Ensures the collision field is fully updated
/// - Calls the lighting prebake system to calculate static lighting
///
/// # Arguments
/// * `bf` - Board data resource for collision/lighting fields
/// * `qt` - Query to access all level entities with behaviors and positions
pub fn load_map_add_prebaked_lighting(
    mut bf: ResMut<BoardData>,
    qt: Query<(Entity, &Position, &Behavior)>,
) {
    // Ensure the collision field is up to date first
    rebuild_collision_data(&mut bf, &qt);

    // Call the prebaking function to calculate static lighting
    prebake_lighting_field(&mut bf, &qt);

    // Log completion
    info!("Map loaded with prebaked lighting data");
}
