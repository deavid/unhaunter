//! This module contains the main systems and plugins for managing the board in the Unhaunter game.
//! It includes systems for applying isometric perspective, rebuilding collision data, and updating
//! the lighting field based on the current state of the board and behaviors.

use bevy::diagnostic::{Diagnostic, DiagnosticPath, RegisterDiagnostic};
use bevy::prelude::*;

use uncore::behavior::Behavior;
use uncore::components::board::position::Position;
use uncore::metric_recorder::SendMetric;
use uncore::resources::roomdb::RoomDB;
use uncore::resources::visibility_data::VisibilityData;
use uncore::{resources::board_data::BoardData, types::board::fielddata::CollisionFieldData};

use crate::board::spritedb::SpriteDB;

pub const APPLY_PERSPECTIVE: DiagnosticPath =
    DiagnosticPath::const_new("unboard/systems/apply_perspective");

/// Main system of board that moves the tiles to their correct place on the screen
/// following the isometric perspective.
///
/// # Arguments
///
/// * `q` - A query for entities with `Position` and `Transform` components that have changed.
pub fn apply_perspective(mut q: Query<(&Position, &mut Transform), Changed<Position>>) {
    let measure = APPLY_PERSPECTIVE.time_measure();

    for (pos, mut transform) in q.iter_mut() {
        transform.translation = pos.to_screen_coord();
    }

    measure.end_ms();
}

/// Plugin for initializing board-related resources and systems.
pub struct UnhaunterBoardPlugin;

impl Plugin for UnhaunterBoardPlugin {
    /// Builds the plugin by initializing resources and adding systems and events.
    ///
    /// # Arguments
    ///
    /// * `app` - A mutable reference to the Bevy app.
    fn build(&self, app: &mut App) {
        app.init_resource::<BoardData>()
            .init_resource::<VisibilityData>()
            .init_resource::<SpriteDB>()
            .init_resource::<RoomDB>()
            .add_systems(Update, apply_perspective);
        app.register_diagnostic(Diagnostic::new(APPLY_PERSPECTIVE).with_suffix("ms"));
    }
}

/// Rebuilds the collision data for the board based on the current state of the board and behaviors.
///
/// # Arguments
///
/// * `bf` - A mutable reference to the `BoardData` resource, which stores the collision field.
/// * `qt` - A query for entities with `Position` and `Behavior` components.
pub fn rebuild_collision_data(bf: &mut ResMut<BoardData>, qt: &Query<(&Position, &Behavior)>) {
    // info!("Collision rebuild");
    assert_eq!(
        bf.collision_field.shape(),
        [bf.map_size.0, bf.map_size.1, bf.map_size.2]
    );
    bf.collision_field.fill(CollisionFieldData::default());

    for (pos, _behavior) in qt.iter().filter(|(_p, b)| b.p.movement.walkable) {
        let bpos = pos.to_board_position();
        let colfd = CollisionFieldData {
            player_free: true,
            ghost_free: true,
            see_through: false,
        };
        bf.collision_field[bpos.ndidx()] = colfd;
    }
    for (pos, behavior) in qt.iter().filter(|(_p, b)| b.p.movement.player_collision) {
        let bpos = pos.to_board_position();
        let colfd = CollisionFieldData {
            player_free: false,
            ghost_free: !behavior.p.movement.ghost_collision,
            see_through: behavior.p.light.see_through,
        };
        bf.collision_field[bpos.ndidx()] = colfd;
    }
}

// rebuild_lighting_field function has been moved to unlight::lighting
