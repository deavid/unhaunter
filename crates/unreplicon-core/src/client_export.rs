use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Generic client-to-server component upload message.
///
/// Replaces the closed-enum `ExportGearStateMessage` / `GearSkeletonState` and
/// the flat-struct `ExportStateMessage` with a single generic transport. Each
/// concrete instantiation (`ExportClientComponent<Flashlight>`,
/// `ExportClientComponent<Position>`, …) is an independent replicon message
/// registered by the **owning domain plugin**, not by `unreplicon-plugin`.
///
/// # Registration in domain plugins
///
/// ```rust,no_run,ignore
/// // In the domain plugin's app_setup:
/// app.add_mapped_client_message::<ExportClientComponent<Flashlight>>(Channel::Unreliable);
/// ```
///
/// # Phase 2 (future)
/// A generic helper will generate the export/import boilerplate so each domain
/// only needs a one-line registration call. Until then, each domain writes its
/// own send (client) and receive (server) systems manually.
// TODO(Phase 2): introduce a macro or blanket helper to reduce per-type boilerplate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportClientComponent<T> {
    /// The server-side entity whose component is being uploaded.
    /// Automatically translated from client-side ID by
    /// `add_mapped_client_message`.
    pub entity: Entity,
    /// Snapshot of the component value from the client's local simulation.
    pub data: T,
}

// Manual `Message` impl: the proc-macro derive would add `T: Message` which is wrong here.
// We only need `T: Send + Sync + 'static` (the base `Message` requirements).
impl<T: Send + Sync + 'static> Message for ExportClientComponent<T> {}

impl<T: Send + Sync + 'static> bevy::ecs::entity::MapEntities for ExportClientComponent<T> {
    fn map_entities<M: bevy::ecs::entity::EntityMapper>(&mut self, mapper: &mut M) {
        self.entity = mapper.get_mapped(self.entity);
    }
}
