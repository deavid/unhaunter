use bevy::prelude::*;

use crate::npchelp;

pub struct UnhaunterNPCPlugin;

impl Plugin for UnhaunterNPCPlugin {
    fn build(&self, app: &mut App) {
        npchelp::app_setup(app);
    }
}
