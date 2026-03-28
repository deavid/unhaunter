use crate::systems;
use bevy::prelude::*;
use bevy_replicon::prelude::AppMarkerExt;
use unbehavior_core::components::{FloorItemCollidable, TmxEntityId};
use uninteraction_core::events::RoomStateSyncEvent;
use uninteraction_core::interaction::ExecuteInteractionEvent;
use uninteraction_core::interaction::Toggleable;
use unreplicon_core::noop::{noop_remove, noop_write};
use unreplicon_core::ownership::LocallyOwned;

pub struct UnhaunterInteractionCorePlugin;

pub struct UnhaunterInteractionClientPlugin;

impl Plugin for UnhaunterInteractionCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ExecuteInteractionEvent>()
            .add_message::<RoomStateSyncEvent>();
        app.set_marker_fns::<LocallyOwned, Toggleable>(noop_write::<Toggleable>, noop_remove);
        app.set_marker_fns::<LocallyOwned, FloorItemCollidable>(
            noop_write::<FloorItemCollidable>,
            noop_remove,
        );
        app.set_marker_fns::<LocallyOwned, TmxEntityId>(noop_write::<TmxEntityId>, noop_remove);
        systems::app_setup_core(app);
    }
}

impl Plugin for UnhaunterInteractionClientPlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup_client(app);
    }
}
