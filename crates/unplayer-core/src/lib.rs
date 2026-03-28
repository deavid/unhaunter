pub mod assets;
pub mod colors;
pub mod components;

pub mod authoritative {
    use bevy::prelude::*;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct PlayerAuthoritativeLogicSet;
}
