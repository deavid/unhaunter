use bevy::prelude::*;
use uncore::{events::walkie::WalkieEvent, resources::walkie::WalkiePlay};

use crate::walkie_play::{on_game_load, player_forgot_equipment, walkie_talk};

pub struct UnhaunterWalkiePlugin;

impl Plugin for UnhaunterWalkiePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WalkieEvent>()
            .init_resource::<WalkiePlay>()
            .add_systems(Update, player_forgot_equipment)
            .add_systems(Update, walkie_talk)
            .add_systems(Update, on_game_load);
    }
}
