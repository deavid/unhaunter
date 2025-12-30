use super::systems;
use bevy::prelude::*;
use unplayer_core::resources::PlayerState;

pub struct UnhaunterPlayerPlugin;

impl Plugin for UnhaunterPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerState>();
        systems::app_setup(app);
    }
}
