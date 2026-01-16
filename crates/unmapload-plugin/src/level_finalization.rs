//! # Level Finalization Module
//!
//! This module handles post-load processing for levels, including:
//! - Temperature field initialization and smoothing
//! - Processing mesh placeholders into actual mesh instances
//! - Adding prebaked lighting to the level
//! - Computing usable area statistics for the level

use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use unbehavior::behavior::Behavior;
use unbehavior::roomdb::RoomDB;
use unboard_core::resources::board_topology::BoardTopology;
use unevents_core::events::loadlevel::LevelReadyEvent;
use unevents_core::events::roomchanged::RoomChangedEvent;
use unlight_plugin::lighting_sim::systems::prebake_lighting_field;
use unlight_plugin::resources::light_grid::LightGrid;
use unrender_std::board::tiledata::PreMesh;
use unrender_std::utils::collision::rebuild_collision_data;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;
use untypes_core::states::{AppState, GameState};

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
fn after_level_ready(
    bf: Res<BoardTopology>,
    mut ev: MessageReader<LevelReadyEvent>,
    mut ev_room: MessageWriter<RoomChangedEvent>,
    roomdb: Res<RoomDB>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if ev.is_empty() {
        return;
    }

    // Get level parameters
    let open_van = ev.read().next().unwrap().open_van;

    // Switch to in-game state
    next_app_state.set(AppState::InGame);
    next_game_state.set(GameState::None);

    // Send room changed event with van open state
    ev_room.write(RoomChangedEvent::init(open_van));

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
                    let floor_rooms = rooms_per_floor.entry(bpos.z).or_default();
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
fn process_pre_meshes(
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
                    let base_quad = Mesh::from(unrender_std::utils::quadcc::QuadCC::new(
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
fn load_map_add_prebaked_lighting(
    mut bf: ResMut<BoardTopology>,
    mut lg: ResMut<LightGrid>,
    qt: Query<(Entity, &Position, &Behavior)>,
    _roomdb: Res<RoomDB>,
) {
    // Ensure the collision field is up to date first
    rebuild_collision_data(&mut bf, &qt);

    // Call the prebaking function to calculate static lighting
    prebake_lighting_field(&mut bf, &mut lg, &qt);

    // Log completion
    info!("Map loaded with prebaked lighting data");
}

pub(crate) fn app_setup(app: &mut App) {
    use bevy::prelude::on_message;
    use unevents_core::events::loadlevel::LevelReadyEvent;
    app.add_systems(Update, (process_pre_meshes, after_level_ready))
        .add_systems(
            Update,
            load_map_add_prebaked_lighting.run_if(on_message::<LevelReadyEvent>),
        );
}
