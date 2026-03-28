pub mod components;
pub mod resources;
pub mod states;

use bevy::prelude::*;

/// System set for all input collection systems.
/// All systems that populate PlayerInput should run in this set.
/// Other systems should order themselves `.after(PlayerInputSet)` to ensure input is available.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PlayerInputSet;
