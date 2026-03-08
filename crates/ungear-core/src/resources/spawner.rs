use crate::types::gear::kind::GearKind;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use serde::{Deserialize, Serialize};
use unfoundation_core::types::gear::VisualKey;
use unspatial_core::position::Position;

/// A marker component for all gear entities.
#[derive(Component, Debug, Clone, Copy, Reflect, Default, Serialize, Deserialize)]
#[reflect(Component)]
pub struct GearMarker;

/// Marker inserted once a gear entity has been fully hydrated with type-specific components.
/// On authority nodes this is never needed (components are inserted at spawn time).
/// On join clients this is inserted by `hydrate_gear_system` after the gear builder runs.
#[derive(Component, Debug, Default)]
pub struct GearHydrated;

/// Metadata for a piece of gear.
#[derive(Clone, Debug)]
pub struct GearMetadata {
    pub name: String,
    pub description: String,
    pub sprite_idx: VisualKey,
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

    /// Applies type-specific components to an already-existing gear entity.
    /// Used by `hydrate_gear_system` on join clients after replication delivers the entity.
    pub fn hydrate(&self, commands: &mut Commands, entity: Entity, kind: GearKind) {
        let mut entity_cmd = commands.entity(entity);
        if let Some(builder) = self.builders.get(&kind) {
            (builder)(&mut entity_cmd);
        } else {
            warn!("No gear builder registered for {:?} during hydration", kind);
        }
    }
}
