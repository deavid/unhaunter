use bevy::prelude::*;

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
