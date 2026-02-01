use bevy::prelude::*;
use enum_iterator::all;
use std::collections::HashMap;
use unfoundation_core::types::gear::VisualKey;
use ungear_core::types::gear::sprite_id::GearSpriteID;

#[derive(Resource, Debug, Default, Clone)]
pub struct SpriteRegistry {
    pub registry: HashMap<String, usize>,
}

impl SpriteRegistry {
    pub fn get(&self, key: &VisualKey) -> usize {
        *self.registry.get(key.as_str()).unwrap_or_else(|| {
            warn!("VisualKey not found in registry: {}", key);
            self.registry.get(VisualKey::NONE).unwrap_or(&0)
        })
    }
}

pub fn setup_sprite_registry(mut commands: Commands) {
    let mut registry = HashMap::new();

    // Populate from legacy GearSpriteID
    for id in all::<GearSpriteID>() {
        registry.insert(id.to_visual_key().as_str().to_string(), id as usize);
    }

    commands.insert_resource(SpriteRegistry { registry });
}
