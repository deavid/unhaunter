use bevy::prelude::*;

#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub enum SpriteType {
    Ghost,
    Breach,
    Player,
    Miasma,
    #[default]
    Other,
}
