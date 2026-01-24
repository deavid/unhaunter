use bevy::prelude::*;

use crate::npchelp;

pub struct UnhaunterNPCPlugin;

impl Plugin for UnhaunterNPCPlugin {
    fn build(&self, app: &mut App) {
        crate::hydration::app_setup(app);
        npchelp::app_setup(app);
    }
}
