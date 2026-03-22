pub mod assets;
pub mod components;

use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PlayerInputSet;

pub mod authoritative {
    use bevy::prelude::*;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct PlayerAuthoritativeLogicSet;
}
