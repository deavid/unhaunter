// Import the systems module
use super::systems;
use bevy::prelude::*;
use uncore::states::GameState;
use uncore::systems::animation::animate_sprite;

pub struct UnhaunterPlayerPlugin;

impl Plugin for UnhaunterPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_sprite.run_if(in_state(GameState::None)));

        // Call the app_setup from the systems module
        systems::app_setup(app);
    }
}
