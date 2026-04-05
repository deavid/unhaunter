use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use uncommon_states_core::UIContextState;
use unnpc_core::events::NpcHelpEvent;

use crate::assets::NpcAssets;
use crate::npchelp;

pub struct UnhaunterNPCCorePlugin;

impl Plugin for UnhaunterNPCCorePlugin {
    fn build(&self, app: &mut App) {
        crate::hydration::app_setup(app);
    }
}

pub struct UnhaunterNPCPlugin;

impl Plugin for UnhaunterNPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(UIContextState::EngineBoot).load_collection::<NpcAssets>(),
        );
        app.add_message::<NpcHelpEvent>();
        npchelp::app_setup(app);
    }
}
