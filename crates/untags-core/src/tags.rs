use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component, Default)]
pub struct PlayerTag;

#[derive(Component, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component, Default)]
pub struct GhostTag;

#[derive(Component, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component, Default)]
pub struct InteractableTag;

#[derive(Component, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component, Default)]
pub struct GearTag;

#[derive(Component, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component, Default)]
pub struct NpcTag;

#[derive(Component, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component, Default)]
pub struct TruckTag;
