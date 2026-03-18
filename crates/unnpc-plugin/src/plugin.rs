use bevy::prelude::*;
use unnpc_core::events::NpcHelpEvent;

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
        app.add_message::<NpcHelpEvent>();
        npchelp::app_setup(app);
    }
}
