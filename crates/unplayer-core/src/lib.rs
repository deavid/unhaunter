pub mod assets;
pub mod components;

pub mod authoritative {
    use bevy::prelude::*;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct PlayerAuthoritativeLogicSet;
}
