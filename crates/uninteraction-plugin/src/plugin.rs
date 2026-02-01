use crate::systems;
use bevy::prelude::*;
use uninteraction_core::interaction::ExecuteInteractionEvent;

pub struct UnhaunterInteractionPlugin;

impl Plugin for UnhaunterInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ExecuteInteractionEvent>();
        systems::app_setup(app);
    }
}
