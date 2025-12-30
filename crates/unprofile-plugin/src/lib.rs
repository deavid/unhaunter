pub mod plugin;

mod dev_tools;

use bevy::prelude::*;

pub struct UnprofilePlugin;

impl Plugin for UnprofilePlugin {
    fn build(&self, _app: &mut App) {}
}
