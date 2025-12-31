//! This module contains the main systems and plugins for managing the board in the Unhaunter game.
//! It includes systems for applying isometric perspective, rebuilding collision data, and updating
//! the lighting field based on the current state of the board and behaviors.

use bevy::diagnostic::{Diagnostic, DiagnosticPath, RegisterDiagnostic};
use bevy::prelude::*;

use unrender_std::resources::visibility_data::VisibilityData;
use unboard_core::resources::board_data::BoardData;
use unboard_core::resources::roomdb::RoomDB;
use unmetrics_core::SendMetric;
use unspatial_core::Position;

use unrender_std::board::spritedb::SpriteDB;

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
        crate::systems::animation::app_setup(app);
        crate::systems::board_sync::app_setup(app);
        app.init_resource::<BoardData>()
            .init_resource::<VisibilityData>()
            .init_resource::<SpriteDB>()
            .init_resource::<RoomDB>()
            .add_systems(Update, apply_perspective);
        app.register_diagnostic(Diagnostic::new(APPLY_PERSPECTIVE).with_suffix("ms"));
    }
}
