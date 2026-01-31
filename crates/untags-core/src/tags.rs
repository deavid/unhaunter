use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NetworkId(pub u32);

#[derive(Component)]
pub struct PlayerTag {
    pub id: usize,
}

#[derive(Component)]
pub struct GhostTag;

#[derive(Component)]
pub struct InteractableTag;

#[derive(Component)]
pub struct GearTag;

#[derive(Component)]
pub struct NpcTag;

#[derive(Component)]
pub struct TruckTag;
