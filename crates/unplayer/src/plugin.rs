use super::systems;
use bevy::prelude::*;
use uncore_resources::resources::player_state::PlayerState;

pub struct UnhaunterPlayerPlugin;

impl Plugin for UnhaunterPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerState>();
        systems::app_setup(app);
    }
}
