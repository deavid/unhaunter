use bevy::prelude::*;

#[derive(Clone, Debug, Event)]
pub struct NpcHelpEvent {
    pub entity: Entity,
}

impl NpcHelpEvent {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
