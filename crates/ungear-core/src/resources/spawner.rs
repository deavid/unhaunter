use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use uncore_foundation::types::gear::{GearKind, GearSpriteID};
use unspatial_core::Position;

/// A marker component for all gear entities.
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct GearMarker;

/// Metadata for a piece of gear.
#[derive(Clone, Debug)]
pub struct GearMetadata {
    pub name: String,
    pub description: String,
    pub sprite_idx: GearSpriteID,
}

/// A registry that knows how to spawn entities for each GearKind.
#[derive(Resource, Default)]
pub struct GearSpawnerRegistry {
    /// Maps a GearKind to a function that adds components to an entity.
    pub builders: HashMap<GearKind, Box<dyn Fn(&mut EntityCommands) + Send + Sync>>,
    /// Maps a GearKind to its metadata.
    pub metadata: HashMap<GearKind, GearMetadata>,
}

impl GearSpawnerRegistry {
    /// Registers a builder function and metadata for a specific GearKind.
    pub fn register<F>(&mut self, kind: GearKind, metadata: GearMetadata, builder: F)
    where
        F: Fn(&mut EntityCommands) + Send + Sync + 'static,
    {
        self.builders.insert(kind, Box::new(builder));
        self.metadata.insert(kind, metadata);
    }

    /// Registers metadata for a specific GearKind.
    pub fn register_metadata(&mut self, kind: GearKind, metadata: GearMetadata) {
        self.metadata.insert(kind, metadata);
    }

    /// Spawns a new gear entity for the given GearKind.
    pub fn spawn(&self, commands: &mut Commands, kind: GearKind) -> Entity {
        let mut entity_cmd = commands.spawn((GearMarker, kind, Position::new_i64(0, 0, 0)));
        if let Some(builder) = self.builders.get(&kind) {
            (builder)(&mut entity_cmd);
        } else {
            warn!("No gear builder registered for {:?}", kind);
        }
        entity_cmd.id()
    }
}
