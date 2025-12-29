use bevy::prelude::*;

#[derive(Debug, Resource, Clone, Default)]
pub struct TruckGear {
    pub inventory: Vec<Entity>,
}
