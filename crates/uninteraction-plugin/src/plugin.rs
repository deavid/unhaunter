use crate::systems;
use bevy::prelude::*;
use unevents_core::events::roomchanged::RoomStateSyncEvent;
use uninteraction_core::interaction::ExecuteInteractionEvent;

pub struct UnhaunterInteractionCorePlugin;

impl Plugin for UnhaunterInteractionCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ExecuteInteractionEvent>()
            .add_message::<RoomStateSyncEvent>();
        systems::app_setup(app);
    }
}
