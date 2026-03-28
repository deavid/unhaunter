//! Generic helpers for client-to-server component export.
//!
//! Provides an [`App`] extension trait with three methods that register a full
//! send+import pipeline for a component type in a single call, eliminating
//! the per-type boilerplate that would otherwise live in domain plugin
//! `net_state` modules.
//!
//! # Patterns
//!
//! - [`AppClientExportExt::add_component_export`] — component lives directly
//!   on entities matched by an anchor marker (e.g. `PlayerSprite`). Registers
//!   both send and import systems alongside echo-suppression (`set_marker_fns`).
//!
//! - [`AppClientExportExt::add_gear_component_export`] — component lives on
//!   gear entities iterated via [`PlayerGear`]. Import is a plain clone-write
//!   (no secondary component writes, no derived fields). Send runs only on
//!   pure join clients (`is_pure_client`).
//!
//! - [`AppClientExportExt::add_gear_component_send`] — like the gear variant
//!   above but registers only the send side. Use when the import system needs
//!   custom logic (e.g. mirroring a value to a secondary component, or
//!   re-deriving a field server-side). The caller registers its own import.
//!
//! # Gear echo-suppression (`set_marker_fns`)
//!
//! The gear helper methods do **not** call `set_marker_fns`. Echo suppression
//! for known gear item types is registered centrally by
//! `unreplicon-plugin/replication/gearitems.rs`. Calling `set_marker_fns`
//! for the same type twice would panic bevy_replicon at startup.

use bevy::ecs::component::Mutable;
use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppMarkerExt, Channel, ClientId, ClientMessageAppExt, FromClient, Replicated,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use ungear_core::components::playergear::PlayerGear;
use untypes_core::roles::{AuthorityRole, is_pure_client};
use untypes_core::states::AppState;

use crate::client_export::ExportClientComponent;
use crate::noop::{noop_remove, noop_write};
use crate::ownership::{LocallyOwned, Owner, OwnerId};

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn owner_to_client(id: OwnerId) -> ClientId {
    match id {
        OwnerId::Server => ClientId::Server,
        OwnerId::Client(e) => ClientId::Client(e),
    }
}

fn gear_slots(gear: &PlayerGear) -> impl Iterator<Item = Entity> + '_ {
    [gear.left_hand, gear.right_hand]
        .into_iter()
        .flatten()
        .chain(gear.inventory.iter().copied())
}

// ---------------------------------------------------------------------------
// Generic system functions
// ---------------------------------------------------------------------------

/// Send system: reads `T` from locally-owned entities that also have `Anchor`,
/// and writes an [`ExportClientComponent<T>`] message for each one.
fn send_component<T: Component + Clone, Anchor: Component>(
    q_local: Query<(Entity, &T), (With<LocallyOwned>, With<Anchor>)>,
    mut writer: MessageWriter<ExportClientComponent<T>>,
) {
    for (entity, comp) in q_local.iter() {
        writer.write(ExportClientComponent {
            entity,
            data: comp.clone(),
        });
    }
}

/// Import system: reads incoming [`ExportClientComponent<T>`] messages and
/// applies them to remote-owned entities that also have `Anchor`.
fn import_component<T: Component<Mutability = Mutable> + Clone, Anchor: Component>(
    mut reader: MessageReader<FromClient<ExportClientComponent<T>>>,
    mut q: Query<(&Owner, &mut T), (Without<LocallyOwned>, With<Anchor>)>,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut comp)) = q.get_mut(entity) else {
            warn!(
                "import_component<{}>: entity {:?} not found",
                std::any::type_name::<T>(),
                entity
            );
            continue;
        };
        if owner_to_client(owner.0) != msg.client_id {
            warn!(
                "import_component<{}>: ownership mismatch for {:?}",
                std::any::type_name::<T>(),
                entity
            );
            continue;
        }
        *comp = msg.message.data.clone();
    }
}

/// Send system for gear components: iterates gear entity slots via [`PlayerGear`]
/// on the locally-owned player entity and emits one message per gear item that
/// has component `T`.
fn send_gear_component<T: Component + Clone>(
    q_player: Query<&PlayerGear, With<LocallyOwned>>,
    q_comp: Query<&T>,
    mut writer: MessageWriter<ExportClientComponent<T>>,
) {
    for gear in q_player.iter() {
        for entity in gear_slots(gear) {
            if let Ok(comp) = q_comp.get(entity) {
                writer.write(ExportClientComponent {
                    entity,
                    data: comp.clone(),
                });
            }
        }
    }
}

/// Import system for gear components with uniform clone-write semantics.
fn import_gear_component<T: Component<Mutability = Mutable> + Clone>(
    mut reader: MessageReader<FromClient<ExportClientComponent<T>>>,
    mut q_gear: Query<(&Owner, &mut T), (Without<LocallyOwned>, With<Replicated>)>,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut comp)) = q_gear.get_mut(entity) else {
            warn!(
                "import_gear_component<{}>: entity {:?} not found",
                std::any::type_name::<T>(),
                entity
            );
            continue;
        };
        if owner_to_client(owner.0) != msg.client_id {
            warn!(
                "import_gear_component<{}>: ownership mismatch for {:?}",
                std::any::type_name::<T>(),
                entity
            );
            continue;
        }
        *comp = msg.message.data.clone();
    }
}

// ---------------------------------------------------------------------------
// Extension trait
// ---------------------------------------------------------------------------

/// Extension methods on [`App`] for registering client-to-server component
/// export pipelines with minimal boilerplate.
///
/// # Phase 2
///
/// This trait is the Phase 2 boilerplate reduction described in
/// `docs/ddd-analysis/04_replicon_client_export.md`. Each domain plugin calls
/// a single method per exported component type instead of writing three
/// identical functions (send, import, registration).
pub trait AppClientExportExt {
    /// Register a full send + import pipeline for component `T` on entities
    /// that also carry the `Anchor` marker component.
    ///
    /// Registers:
    /// - `add_mapped_client_message::<ExportClientComponent<T>>(Channel::Unreliable)`
    /// - `set_marker_fns::<LocallyOwned, T>(noop_write, noop_remove)` (echo suppression)
    /// - Send system: runs in `AppState::InGame` on pure join clients only (`is_pure_client`)
    /// - Import system: runs in `AppState::InGame` when `AuthorityRole` exists
    ///
    /// Typical call:
    /// ```rust,ignore
    /// app.add_component_export::<Stamina, PlayerSprite>();
    /// ```
    fn add_component_export<T, Anchor>(&mut self) -> &mut Self
    where
        T: Component<Mutability = Mutable> + Clone + Serialize + DeserializeOwned,
        Anchor: Component;

    /// Register a full send + import pipeline for a gear component iterated
    /// via [`PlayerGear`].
    ///
    /// Import is a plain clone-write (no secondary component writes, no
    /// derived fields). For types with custom import logic, use
    /// [`add_gear_component_send`](AppClientExportExt::add_gear_component_send)
    /// and register the import system manually.
    ///
    /// Send runs only on pure join clients (`is_pure_client`) to avoid the
    /// authority overwriting its own ground-truth gear state.
    ///
    /// **Does not** call `set_marker_fns`; echo suppression for known gear
    /// types is registered centrally in `unreplicon-plugin/replication/gearitems.rs`.
    fn add_gear_component_export<T>(&mut self) -> &mut Self
    where
        T: Component<Mutability = Mutable> + Clone + Serialize + DeserializeOwned;

    /// Register only the send side for a gear component.
    ///
    /// Registers the message and the generic send system. The caller must
    /// register its own import system when the import needs custom logic
    /// (e.g. mirroring a value to `Toggleable.is_on`, re-deriving `active`).
    ///
    /// **Does not** call `set_marker_fns`.
    fn add_gear_component_send<T>(&mut self) -> &mut Self
    where
        T: Component<Mutability = Mutable> + Clone + Serialize + DeserializeOwned;
}

impl AppClientExportExt for App {
    fn add_component_export<T, Anchor>(&mut self) -> &mut Self
    where
        T: Component<Mutability = Mutable> + Clone + Serialize + DeserializeOwned,
        Anchor: Component,
    {
        self.add_mapped_client_message::<ExportClientComponent<T>>(Channel::Unreliable);
        self.set_marker_fns::<LocallyOwned, T>(noop_write::<T>, noop_remove);
        self.add_systems(
            Update,
            send_component::<T, Anchor>
                .run_if(in_state(AppState::InGame))
                .run_if(is_pure_client),
        );
        self.add_systems(
            Update,
            import_component::<T, Anchor>
                .run_if(in_state(AppState::InGame))
                .run_if(resource_exists::<AuthorityRole>),
        );
        self
    }

    fn add_gear_component_export<T>(&mut self) -> &mut Self
    where
        T: Component<Mutability = Mutable> + Clone + Serialize + DeserializeOwned,
    {
        self.add_mapped_client_message::<ExportClientComponent<T>>(Channel::Unreliable);
        self.add_systems(
            Update,
            send_gear_component::<T>
                .run_if(in_state(AppState::InGame))
                .run_if(is_pure_client),
        );
        self.add_systems(
            Update,
            import_gear_component::<T>
                .run_if(in_state(AppState::InGame))
                .run_if(resource_exists::<AuthorityRole>),
        );
        self
    }

    fn add_gear_component_send<T>(&mut self) -> &mut Self
    where
        T: Component<Mutability = Mutable> + Clone + Serialize + DeserializeOwned,
    {
        self.add_mapped_client_message::<ExportClientComponent<T>>(Channel::Unreliable);
        self.add_systems(
            Update,
            send_gear_component::<T>
                .run_if(in_state(AppState::InGame))
                .run_if(is_pure_client),
        );
        self
    }
}
