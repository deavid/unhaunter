//! # Level Finalization Module
//!
//! This module handles post-load processing for levels, including:
//! - Temperature field initialization and smoothing
//! - Processing mesh placeholders into actual mesh instances
//! - Rebuilding collision data
//! - Computing usable area statistics for the level

use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use unbehavior_core::behavior::Behavior;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use unboard_core::resources::roomdb::RoomTopology;
use unboard_core::utils::rebuild_collision_data;
use uninput_core::states::InGameUiState;
use uninteraction_core::events::{RoomChangedEvent, RoomStateSyncEvent};
use unmission_core::events::LevelReadyEvent;
use unmission_core::types::SimulationState;
use unorchestrator_core::UIContextState;
use unrender_std::board::tiledata::PreMesh;
use unrender_std::components::visuals::ResolutionFactor;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;

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
/// * `room_topology` - Room topology resource for room information
/// * `next_game_state` - State machine to transition to in-game state
fn after_level_ready(
    bcf: Res<BoardCollisionField>,
    mut ev: MessageReader<LevelReadyEvent>,
    mut ev_room: MessageWriter<RoomChangedEvent>,
    mut ev_room_sync: MessageWriter<RoomStateSyncEvent>,
    room_topology: Res<RoomTopology>,
    mut next_app_state: ResMut<NextState<UIContextState>>,
    mut next_game_state: ResMut<NextState<InGameUiState>>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
) {
    if ev.is_empty() {
        return;
    }

    next_sim_state.set(SimulationState::Spawning);

    // Get level parameters
    let open_van = ev.read().next().unwrap().open_van;

    // Switch to in-game state
    next_app_state.set(UIContextState::InGame);
    next_game_state.set(InGameUiState::Running);

    // Send synchronization events
    ev_room_sync.write(RoomStateSyncEvent);
    ev_room.write(RoomChangedEvent::init(open_van));

    // --- Calculate Usable Haunted Area ---
    let mut total_usable_area_m2 = 0.0;
    let mut area_per_floor: HashMap<i64, f32> = HashMap::new();
    let mut rooms_per_floor: HashMap<i64, HashMap<String, f32>> = HashMap::new();
    let tile_area = BoardPosition::area_per_tile_m2();

    // Calculate usable area for each floor and room (excluding solid walls)
    for bpos in room_topology.room_tiles.keys() {
        if let Some(cf) = bcf.0.get(bpos.ndidx()) {
            // Exclude static walls (opaque and not dynamic)
            if cf.see_through || cf.is_dynamic {
                // Get the floor and increment its area
                let floor_area = area_per_floor.entry(bpos.z).or_insert(0.0);
                *floor_area += tile_area;

                // Add to room area calculation, organized by floor
                if let Some(room_name) = room_topology.room_tiles.get(bpos) {
                    let floor_rooms = rooms_per_floor.entry(bpos.z).or_default();
                    let room_area = floor_rooms.entry(room_name.clone()).or_insert(0.0);
                    *room_area += tile_area;
                }

                total_usable_area_m2 += tile_area;
            }
        } else {
            warn!(
                "Tile at {:?} found in RoomTopology but not in behavior_field.",
                bpos
            );
        }
    }

    // Log area statistics for each floor and its rooms
    trace!("--- Usable Haunted Area Calculation ---");
    let mut sorted_floors: Vec<_> = area_per_floor.keys().collect();
    sorted_floors.sort();

    for floor_z in sorted_floors {
        trace!("--- Floor {} ---", floor_z);

        // Print rooms for this floor
        if let Some(floor_rooms) = rooms_per_floor.get(floor_z) {
            let mut sorted_rooms: Vec<_> = floor_rooms.keys().collect();
            sorted_rooms.sort();

            for room_name in sorted_rooms {
                if let Some(area) = floor_rooms.get(room_name) {
                    trace!("  Room '{}': {:.2} m²", room_name, area);
                }
            }
        }

        // Print floor total
        if let Some(area) = area_per_floor.get(floor_z) {
            trace!("  Floor {} Total: {:.2} m²", floor_z, area);
        }
    }

    trace!("Grand Total Usable Area: {:.2} m²", total_usable_area_m2);
    trace!("--------------------------------------");
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
    query: Query<(Entity, &PreMesh, &ResolutionFactor)>,
    images: Res<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, pre_mesh, rf) in query.iter() {
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
                    trace!(
                        "Physical image size: {} x {} (Resolution Factor: {})",
                        sz.width, sz.height, rf.0
                    );
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
                    trace!("Processed entity: {:?}", entity);
                }
            }
        }
    }
}

/// Rebuilds collision data after the level is fully loaded.
fn rebuild_collision_on_level_ready(
    bf: Res<BoardTopology>,
    mut bcf: ResMut<BoardCollisionField>,
    qt: Query<(Entity, &Position, &Behavior)>,
) {
    // Ensure the collision field is up to date first
    rebuild_collision_data(&bf, &mut bcf, &qt);

    // Log completion
    debug!("Map collision data rebuilt");
}

pub(crate) fn app_setup(app: &mut App) {
    use unmission_core::events::LevelReadyEvent;

    // Note: process_pre_meshes is registered in app_setup_render_only()
    // which is only called for non-headless clients
    app.add_systems(
        Update,
        (rebuild_collision_on_level_ready, after_level_ready)
            .chain()
            .run_if(bevy::prelude::on_message::<LevelReadyEvent>),
    );
}

/// Register rendering-only systems for level finalization.
/// This should only be called on clients (not headless dedicated server).
pub(crate) fn app_setup_render_only(app: &mut App) {
    app.add_systems(Update, process_pre_meshes);
}
