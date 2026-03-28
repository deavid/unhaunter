//! # Level Finalization Module
//!
//! This module handles post-load processing for levels, including:
//! - Rebuilding collision data
//! - Computing usable area statistics for the level

use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use unbehavior_core::behavior::Behavior;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use unboard_core::resources::roomdb::RoomTopology;
use unboard_core::utils::rebuild_collision_data;
use unmission_core::events::LevelReadyEvent;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;

/// Logs the usable haunted area once the map is fully hydrated.
fn log_usable_area_on_level_ready(
    bcf: Res<BoardCollisionField>,
    mut ev: MessageReader<LevelReadyEvent>,
    room_topology: Res<RoomTopology>,
) {
    if ev.is_empty() {
        return;
    }

    let _ = ev.read().last();

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

    app.add_systems(
        Update,
        (
            rebuild_collision_on_level_ready,
            log_usable_area_on_level_ready,
        )
            .chain()
            .run_if(bevy::prelude::on_message::<LevelReadyEvent>),
    );
}
