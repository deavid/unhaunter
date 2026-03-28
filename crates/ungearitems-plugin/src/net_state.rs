//! Client-to-server gear component export.
//!
//! Each gear component type has a dedicated send system (client) and a matching
//! import system (server). Systems are registered here instead of in
//! `unreplicon-plugin`, keeping domain knowledge inside the gear domain crate.
//!
//! The host/offline authority already has the ground-truth gear state locally.
//! Running send systems on the authority would create a feedback loop where the
//! host's current state is overwritten by a one-frame-stale copy received via
//! the message path. All send systems are therefore gated to `is_pure_client`.
//!
//! # Phase 2 (future)
//! Per-type boilerplate here is a candidate for macro generation once the
//! pattern stabilises. See `ExportClientComponent` docs for details.

use bevy::prelude::*;
use bevy_replicon::prelude::{Channel, ClientId, ClientMessageAppExt, FromClient, Replicated};
use ungear_core::components::playergear::PlayerGear;
use ungearitems_core::components::flashlight::{Flashlight, FlashlightStatus};
use ungearitems_core::components::quartz::QuartzStoneData;
use ungearitems_core::components::redtorch::RedTorch;
use ungearitems_core::components::repellentflask::RepellentFlask;
use ungearitems_core::components::sage::SageBundleData;
use ungearitems_core::components::salt::SaltData;
use ungearitems_core::components::uvtorch::UVTorch;
use uninteraction_core::interaction::Toggleable;
use unorchestrator_core::UIContextState;
use unreplicon_core::client_export::ExportClientComponent;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unreplicon_core::resources::{AuthorityRole, is_pure_client};

pub(crate) fn app_setup(app: &mut App) {
    // Register one ExportClientComponent<T> per gear component type.
    // All use the mapped variant so replicon auto-translates entity IDs.
    app.add_mapped_client_message::<ExportClientComponent<Flashlight>>(Channel::Unreliable);
    app.add_mapped_client_message::<ExportClientComponent<UVTorch>>(Channel::Unreliable);
    app.add_mapped_client_message::<ExportClientComponent<RedTorch>>(Channel::Unreliable);
    app.add_mapped_client_message::<ExportClientComponent<RepellentFlask>>(Channel::Unreliable);
    app.add_mapped_client_message::<ExportClientComponent<SaltData>>(Channel::Unreliable);
    app.add_mapped_client_message::<ExportClientComponent<SageBundleData>>(Channel::Unreliable);
    app.add_mapped_client_message::<ExportClientComponent<QuartzStoneData>>(Channel::Unreliable);
    app.add_mapped_client_message::<ExportClientComponent<Toggleable>>(Channel::Unreliable);

    // Client-side: send gear state every frame.
    // Gated to pure clients — the authority already has ground-truth locally.
    app.add_systems(
        Update,
        (
            send_export_flashlight,
            send_export_uvtorch,
            send_export_redtorch,
            send_export_repellentflask,
            send_export_salt,
            send_export_sage,
            send_export_quartz,
            send_export_toggleable,
        )
            .in_set(ungearitems_core::GearStateExportSet)
            .run_if(in_state(UIContextState::InGame))
            .run_if(is_pure_client),
    );

    // Server-side: receive and apply gear state from clients.
    app.add_systems(
        Update,
        (
            handle_import_flashlight,
            handle_import_uvtorch,
            handle_import_redtorch,
            handle_import_repellentflask,
            handle_import_salt,
            handle_import_sage,
            handle_import_quartz,
            handle_import_toggleable,
        )
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(UIContextState::InGame)),
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn from_owner_id(owner_id: OwnerId) -> ClientId {
    match owner_id {
        OwnerId::Server => ClientId::Server,
        OwnerId::Client(e) => ClientId::Client(e),
    }
}

fn gear_entities(gear: &PlayerGear) -> impl Iterator<Item = Entity> + '_ {
    [gear.left_hand, gear.right_hand]
        .into_iter()
        .flatten()
        .chain(gear.inventory.iter().copied())
}

// ---------------------------------------------------------------------------
// Send systems (pure client only)
// ---------------------------------------------------------------------------

fn send_export_flashlight(
    q_player: Query<&PlayerGear, With<LocallyOwned>>,
    q_comp: Query<&Flashlight>,
    mut writer: MessageWriter<ExportClientComponent<Flashlight>>,
) {
    for gear in q_player.iter() {
        for entity in gear_entities(gear) {
            if let Ok(comp) = q_comp.get(entity) {
                writer.write(ExportClientComponent {
                    entity,
                    data: comp.clone(),
                });
            }
        }
    }
}

fn send_export_uvtorch(
    q_player: Query<&PlayerGear, With<LocallyOwned>>,
    q_comp: Query<&UVTorch>,
    mut writer: MessageWriter<ExportClientComponent<UVTorch>>,
) {
    for gear in q_player.iter() {
        for entity in gear_entities(gear) {
            if let Ok(comp) = q_comp.get(entity) {
                writer.write(ExportClientComponent {
                    entity,
                    data: comp.clone(),
                });
            }
        }
    }
}

fn send_export_redtorch(
    q_player: Query<&PlayerGear, With<LocallyOwned>>,
    q_comp: Query<&RedTorch>,
    mut writer: MessageWriter<ExportClientComponent<RedTorch>>,
) {
    for gear in q_player.iter() {
        for entity in gear_entities(gear) {
            if let Ok(comp) = q_comp.get(entity) {
                writer.write(ExportClientComponent {
                    entity,
                    data: comp.clone(),
                });
            }
        }
    }
}

fn send_export_repellentflask(
    q_player: Query<&PlayerGear, With<LocallyOwned>>,
    q_comp: Query<&RepellentFlask>,
    mut writer: MessageWriter<ExportClientComponent<RepellentFlask>>,
) {
    for gear in q_player.iter() {
        for entity in gear_entities(gear) {
            if let Ok(comp) = q_comp.get(entity) {
                writer.write(ExportClientComponent {
                    entity,
                    data: comp.clone(),
                });
            }
        }
    }
}

fn send_export_salt(
    q_player: Query<&PlayerGear, With<LocallyOwned>>,
    q_comp: Query<&SaltData>,
    mut writer: MessageWriter<ExportClientComponent<SaltData>>,
) {
    for gear in q_player.iter() {
        for entity in gear_entities(gear) {
            if let Ok(comp) = q_comp.get(entity) {
                writer.write(ExportClientComponent {
                    entity,
                    data: comp.clone(),
                });
            }
        }
    }
}

fn send_export_sage(
    q_player: Query<&PlayerGear, With<LocallyOwned>>,
    q_comp: Query<&SageBundleData>,
    mut writer: MessageWriter<ExportClientComponent<SageBundleData>>,
) {
    for gear in q_player.iter() {
        for entity in gear_entities(gear) {
            if let Ok(comp) = q_comp.get(entity) {
                writer.write(ExportClientComponent {
                    entity,
                    data: comp.clone(),
                });
            }
        }
    }
}

fn send_export_quartz(
    q_player: Query<&PlayerGear, With<LocallyOwned>>,
    q_comp: Query<&QuartzStoneData>,
    mut writer: MessageWriter<ExportClientComponent<QuartzStoneData>>,
) {
    for gear in q_player.iter() {
        for entity in gear_entities(gear) {
            if let Ok(comp) = q_comp.get(entity) {
                writer.write(ExportClientComponent {
                    entity,
                    data: comp.clone(),
                });
            }
        }
    }
}

/// Catch-all: sends `Toggleable` for gear entities that have no specialist
/// component (e.g. Thermometer, EMFMeter, Recorder, etc.).
/// Entities covered by a specialist send system are explicitly skipped to
/// avoid duplicate messages.
fn send_export_toggleable(
    q_player: Query<&PlayerGear, With<LocallyOwned>>,
    q_specialist: Query<(), Or<(With<Flashlight>, With<UVTorch>, With<RedTorch>)>>,
    q_comp: Query<&Toggleable>,
    mut writer: MessageWriter<ExportClientComponent<Toggleable>>,
) {
    for gear in q_player.iter() {
        for entity in gear_entities(gear) {
            // Skip entities handled by a specialist send system.
            if q_specialist.get(entity).is_ok() {
                continue;
            }
            if let Ok(comp) = q_comp.get(entity) {
                writer.write(ExportClientComponent {
                    entity,
                    data: *comp,
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Import systems (authority only)
// ---------------------------------------------------------------------------

fn handle_import_flashlight(
    mut reader: MessageReader<FromClient<ExportClientComponent<Flashlight>>>,
    mut q_gear: Query<
        (&Owner, &mut Flashlight, Option<&mut Toggleable>),
        (Without<LocallyOwned>, With<Replicated>),
    >,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut flashlight, toggleable)) = q_gear.get_mut(entity) else {
            warn!("handle_import_flashlight: entity {:?} not found", entity);
            continue;
        };
        if from_owner_id(owner.0) != msg.client_id {
            warn!(
                "handle_import_flashlight: ownership mismatch for {:?}",
                entity
            );
            continue;
        }
        let is_on = msg.message.data.status != FlashlightStatus::Off;
        if flashlight.status != msg.message.data.status {
            info!(
                "RECV: UPDATING FLASHLIGHT: Entity: {:?}, New Status: {:?}",
                entity, msg.message.data.status
            );
        }
        *flashlight = msg.message.data.clone();
        if let Some(mut t) = toggleable {
            t.is_on = is_on;
        }
    }
}

fn handle_import_uvtorch(
    mut reader: MessageReader<FromClient<ExportClientComponent<UVTorch>>>,
    mut q_gear: Query<
        (&Owner, &mut UVTorch, Option<&mut Toggleable>),
        (Without<LocallyOwned>, With<Replicated>),
    >,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut uvtorch, toggleable)) = q_gear.get_mut(entity) else {
            warn!("handle_import_uvtorch: entity {:?} not found", entity);
            continue;
        };
        if from_owner_id(owner.0) != msg.client_id {
            warn!("handle_import_uvtorch: ownership mismatch for {:?}", entity);
            continue;
        }
        let enabled = msg.message.data.enabled;
        *uvtorch = msg.message.data.clone();
        if let Some(mut t) = toggleable {
            t.is_on = enabled;
        }
    }
}

fn handle_import_redtorch(
    mut reader: MessageReader<FromClient<ExportClientComponent<RedTorch>>>,
    mut q_gear: Query<
        (&Owner, &mut RedTorch, Option<&mut Toggleable>),
        (Without<LocallyOwned>, With<Replicated>),
    >,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut redtorch, toggleable)) = q_gear.get_mut(entity) else {
            warn!("handle_import_redtorch: entity {:?} not found", entity);
            continue;
        };
        if from_owner_id(owner.0) != msg.client_id {
            warn!(
                "handle_import_redtorch: ownership mismatch for {:?}",
                entity
            );
            continue;
        }
        let enabled = msg.message.data.enabled;
        *redtorch = msg.message.data.clone();
        if let Some(mut t) = toggleable {
            t.is_on = enabled;
        }
    }
}

fn handle_import_repellentflask(
    mut reader: MessageReader<FromClient<ExportClientComponent<RepellentFlask>>>,
    mut q_gear: Query<(&Owner, &mut RepellentFlask), (Without<LocallyOwned>, With<Replicated>)>,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut flask)) = q_gear.get_mut(entity) else {
            warn!(
                "handle_import_repellentflask: entity {:?} not found",
                entity
            );
            continue;
        };
        if from_owner_id(owner.0) != msg.client_id {
            warn!(
                "handle_import_repellentflask: ownership mismatch for {:?}",
                entity
            );
            continue;
        }
        // Copy fields from the client message; re-derive `active` server-side
        // to ensure it stays consistent regardless of what the client sent.
        *flask = msg.message.data.clone();
        flask.active = flask.qty > 0 && flask.liquid_content.is_some();
    }
}

fn handle_import_salt(
    mut reader: MessageReader<FromClient<ExportClientComponent<SaltData>>>,
    mut q_gear: Query<(&Owner, &mut SaltData), (Without<LocallyOwned>, With<Replicated>)>,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut salt)) = q_gear.get_mut(entity) else {
            warn!("handle_import_salt: entity {:?} not found", entity);
            continue;
        };
        if from_owner_id(owner.0) != msg.client_id {
            warn!("handle_import_salt: ownership mismatch for {:?}", entity);
            continue;
        }
        *salt = msg.message.data.clone();
    }
}

fn handle_import_sage(
    mut reader: MessageReader<FromClient<ExportClientComponent<SageBundleData>>>,
    mut q_gear: Query<(&Owner, &mut SageBundleData), (Without<LocallyOwned>, With<Replicated>)>,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut sage)) = q_gear.get_mut(entity) else {
            warn!("handle_import_sage: entity {:?} not found", entity);
            continue;
        };
        if from_owner_id(owner.0) != msg.client_id {
            warn!("handle_import_sage: ownership mismatch for {:?}", entity);
            continue;
        }
        *sage = msg.message.data.clone();
    }
}

fn handle_import_quartz(
    mut reader: MessageReader<FromClient<ExportClientComponent<QuartzStoneData>>>,
    mut q_gear: Query<(&Owner, &mut QuartzStoneData), (Without<LocallyOwned>, With<Replicated>)>,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut quartz)) = q_gear.get_mut(entity) else {
            warn!("handle_import_quartz: entity {:?} not found", entity);
            continue;
        };
        if from_owner_id(owner.0) != msg.client_id {
            warn!("handle_import_quartz: ownership mismatch for {:?}", entity);
            continue;
        }
        *quartz = msg.message.data.clone();
    }
}

/// Handles `Toggleable` for generic gear types (Thermometer, EMFMeter, etc.)
/// that have no specialist component. Flashlight/UVTorch/RedTorch are excluded
/// on the send side and will not produce these messages.
fn handle_import_toggleable(
    mut reader: MessageReader<FromClient<ExportClientComponent<Toggleable>>>,
    mut q_gear: Query<(&Owner, &mut Toggleable), (Without<LocallyOwned>, With<Replicated>)>,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut toggleable)) = q_gear.get_mut(entity) else {
            warn!("handle_import_toggleable: entity {:?} not found", entity);
            continue;
        };
        if from_owner_id(owner.0) != msg.client_id {
            warn!(
                "handle_import_toggleable: ownership mismatch for {:?}",
                entity
            );
            continue;
        }
        *toggleable = msg.message.data;
    }
}
