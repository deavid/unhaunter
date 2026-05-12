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
use bevy_replicon::prelude::*;
use uncommon_states_core::UIContextState;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use ungearitems_core::components::flashlight::{Flashlight, FlashlightStatus};
use ungearitems_core::components::quartz::QuartzStoneData;
use ungearitems_core::components::redtorch::RedTorch;
use ungearitems_core::components::repellentflask::RepellentFlask;
use ungearitems_core::components::sage::SageBundleData;
use ungearitems_core::components::salt::SaltData;
use ungearitems_core::components::uvtorch::UVTorch;
use ungearitems_core::events::RepellentHitNetMessage;
use unghost_core::components::logic::ghost_sprite::GhostSprite;
use uninteraction_core::interaction::Toggleable;
use unreplicon_core::client_export::ExportClientComponent;
use unreplicon_core::messages::ReplicatedSoundEvent;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unreplicon_core::resources::{AuthorityRole, LocalPlayerRole, is_pure_client};
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;

pub(crate) fn app_setup(app: &mut App) {
    app.add_client_message::<RepellentHitNetMessage>(Channel::Unreliable);

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
            handle_repellent_hits,
        )
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(UIContextState::InGame)),
    );

    // Dedicated server only: drive ghost repellent delta accumulation / decay.
    // On host/single-player this is handled by the authority branch inside
    // repellent_update (which runs because LocalPlayerRole is present there).
    app.add_systems(
        Update,
        repellent_ghost_delta_tick
            .after(handle_repellent_hits)
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(not(resource_exists::<LocalPlayerRole>))
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

fn to_owner_id(client_id: ClientId) -> OwnerId {
    match client_id {
        ClientId::Server => OwnerId::Server,
        ClientId::Client(e) => OwnerId::Client(e),
    }
}

// ---------------------------------------------------------------------------
// Send systems (pure client only)
// ---------------------------------------------------------------------------

// TODO: Optimize/Throttle - Continuous state currently blasting every frame
fn send_export_flashlight(
    q_items: Query<(Entity, &Flashlight), With<LocallyOwned>>,
    mut writer: MessageWriter<ExportClientComponent<Flashlight>>,
) {
    for (entity, comp) in q_items.iter() {
        writer.write(ExportClientComponent {
            entity,
            data: comp.clone(),
        });
    }
}

// TODO: Optimize/Throttle - Continuous state currently blasting every frame
fn send_export_uvtorch(
    q_items: Query<(Entity, &UVTorch), With<LocallyOwned>>,
    mut writer: MessageWriter<ExportClientComponent<UVTorch>>,
) {
    for (entity, comp) in q_items.iter() {
        writer.write(ExportClientComponent {
            entity,
            data: comp.clone(),
        });
    }
}

// TODO: Optimize/Throttle - Continuous state currently blasting every frame
fn send_export_redtorch(
    q_items: Query<(Entity, &RedTorch), With<LocallyOwned>>,
    mut writer: MessageWriter<ExportClientComponent<RedTorch>>,
) {
    for (entity, comp) in q_items.iter() {
        writer.write(ExportClientComponent {
            entity,
            data: comp.clone(),
        });
    }
}

// TODO: Optimize/Throttle - Continuous state currently blasting every frame
fn send_export_repellentflask(
    q_items: Query<(Entity, &RepellentFlask), With<LocallyOwned>>,
    mut writer: MessageWriter<ExportClientComponent<RepellentFlask>>,
) {
    for (entity, comp) in q_items.iter() {
        writer.write(ExportClientComponent {
            entity,
            data: comp.clone(),
        });
    }
}

// TODO: Optimize/Throttle - Continuous state currently blasting every frame
fn send_export_salt(
    q_items: Query<(Entity, &SaltData), With<LocallyOwned>>,
    mut writer: MessageWriter<ExportClientComponent<SaltData>>,
) {
    for (entity, comp) in q_items.iter() {
        writer.write(ExportClientComponent {
            entity,
            data: comp.clone(),
        });
    }
}

// TODO: Optimize/Throttle - Continuous state currently blasting every frame
fn send_export_sage(
    q_items: Query<(Entity, &SageBundleData), With<LocallyOwned>>,
    mut writer: MessageWriter<ExportClientComponent<SageBundleData>>,
) {
    for (entity, comp) in q_items.iter() {
        writer.write(ExportClientComponent {
            entity,
            data: comp.clone(),
        });
    }
}

// TODO: Optimize/Throttle - Continuous state currently blasting every frame
fn send_export_quartz(
    q_items: Query<(Entity, &QuartzStoneData), With<LocallyOwned>>,
    mut writer: MessageWriter<ExportClientComponent<QuartzStoneData>>,
) {
    for (entity, comp) in q_items.iter() {
        writer.write(ExportClientComponent {
            entity,
            data: comp.clone(),
        });
    }
}

/// Catch-all: sends `Toggleable` for gear entities that have no specialist
/// component (e.g. Thermometer, EMFMeter, Recorder, etc.).
/// Entities covered by a specialist send system are explicitly skipped to
/// avoid duplicate messages.
// TODO: Optimize/Throttle - Continuous state currently blasting every frame
fn send_export_toggleable(
    q_items: Query<
        (Entity, &Toggleable),
        (
            With<LocallyOwned>,
            Without<Flashlight>,
            Without<UVTorch>,
            Without<RedTorch>,
        ),
    >,
    mut writer: MessageWriter<ExportClientComponent<Toggleable>>,
) {
    for (entity, comp) in q_items.iter() {
        writer.write(ExportClientComponent {
            entity,
            data: *comp,
        });
    }
}

// ---------------------------------------------------------------------------
// Import systems (authority only)
// ---------------------------------------------------------------------------

fn handle_import_flashlight(
    mut reader: MessageReader<FromClient<ExportClientComponent<Flashlight>>>,
    mut q_gear: Query<
        (
            &Owner,
            &mut Flashlight,
            Option<&mut Toggleable>,
            Option<&Position>,
            Has<InTruck>,
        ),
        (Without<LocallyOwned>, With<Replicated>),
    >,
    mut ev_replicated_sound: MessageWriter<ToClients<ReplicatedSoundEvent>>,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut flashlight, toggleable, pos, in_truck)) = q_gear.get_mut(entity)
        else {
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

            ev_replicated_sound.write(ToClients {
                mode: SendMode::Broadcast,
                message: ReplicatedSoundEvent {
                    sound_file: "sounds/switch-on-1.ogg".to_string(),
                    volume: 1.0,
                    position: pos.map(|p: &Position| [p.x, p.y, p.z]),
                    triggerer: to_owner_id(msg.client_id),
                    is_inside_truck: in_truck,
                },
            });
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
        (
            &Owner,
            &mut UVTorch,
            Option<&mut Toggleable>,
            Option<&Position>,
            Has<InTruck>,
        ),
        (Without<LocallyOwned>, With<Replicated>),
    >,
    mut ev_replicated_sound: MessageWriter<ToClients<ReplicatedSoundEvent>>,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut uvtorch, toggleable, pos, in_truck)) = q_gear.get_mut(entity)
        else {
            warn!("handle_import_uvtorch: entity {:?} not found", entity);
            continue;
        };
        if from_owner_id(owner.0) != msg.client_id {
            warn!("handle_import_uvtorch: ownership mismatch for {:?}", entity);
            continue;
        }
        if uvtorch.enabled != msg.message.data.enabled {
            ev_replicated_sound.write(ToClients {
                mode: SendMode::Broadcast,
                message: ReplicatedSoundEvent {
                    sound_file: "sounds/switch-on-1.ogg".to_string(),
                    volume: 1.0,
                    position: pos.map(|p: &Position| [p.x, p.y, p.z]),
                    triggerer: to_owner_id(msg.client_id),
                    is_inside_truck: in_truck,
                },
            });
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
        (
            &Owner,
            &mut RedTorch,
            Option<&mut Toggleable>,
            Option<&Position>,
            Has<InTruck>,
        ),
        (Without<LocallyOwned>, With<Replicated>),
    >,
    mut ev_replicated_sound: MessageWriter<ToClients<ReplicatedSoundEvent>>,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut redtorch, toggleable, pos, in_truck)) = q_gear.get_mut(entity)
        else {
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
        if redtorch.enabled != msg.message.data.enabled {
            ev_replicated_sound.write(ToClients {
                mode: SendMode::Broadcast,
                message: ReplicatedSoundEvent {
                    sound_file: "sounds/switch-on-1.ogg".to_string(),
                    volume: 1.0,
                    position: pos.map(|p: &Position| [p.x, p.y, p.z]),
                    triggerer: to_owner_id(msg.client_id),
                    is_inside_truck: in_truck,
                },
            });
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
    mut q_gear: Query<
        (&Owner, &mut Toggleable, Option<&Position>, Has<InTruck>),
        (Without<LocallyOwned>, With<Replicated>),
    >,
    mut ev_replicated_sound: MessageWriter<ToClients<ReplicatedSoundEvent>>,
) {
    for msg in reader.read() {
        let entity = msg.message.entity;
        let Ok((owner, mut toggleable, pos, in_truck)) = q_gear.get_mut(entity) else {
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
        if toggleable.is_on != msg.message.data.is_on {
            ev_replicated_sound.write(ToClients {
                mode: SendMode::Broadcast,
                message: ReplicatedSoundEvent {
                    sound_file: "sounds/switch-on-1.ogg".to_string(),
                    volume: 1.0,
                    position: pos.map(|p: &Position| [p.x, p.y, p.z]),
                    triggerer: to_owner_id(msg.client_id),
                    is_inside_truck: in_truck,
                },
            });
        }
        *toggleable = msg.message.data;
    }
}

// ---------------------------------------------------------------------------
// Repellent hit forwarding (dedicated server)
// ---------------------------------------------------------------------------

/// Server side: receives accumulated repellent hit data sent by pure join
/// clients each frame that their local particles register contact with the
/// ghost. Adds the reported amounts to the ghost's frame accumulators so the
/// authority `GhostSprite` reflects the actual damage.
///
/// Runs on all authority nodes. On host/single-player, no messages will arrive
/// from the pure-client path (there are no pure clients), so this is a no-op.
fn handle_repellent_hits(
    mut reader: MessageReader<FromClient<RepellentHitNetMessage>>,
    mut q_ghost: Query<&mut GhostSprite>,
) {
    for msg in reader.read() {
        let hits = msg.message.hits_this_frame;
        let misses = msg.message.misses_this_frame;
        for mut ghost in q_ghost.iter_mut() {
            ghost.repellent_hits_frame += hits;
            ghost.repellent_misses_frame += misses;
        }
        debug!(
            "handle_repellent_hits: applied hits={:.3} misses={:.3} from client {:?}",
            hits, misses, msg.client_id
        );
    }
}

/// Dedicated-server only: drives ghost repellent delta accumulation and decay
/// every frame. On host/single-player the equivalent logic runs inside the
/// authority branch of `repellent_update` (which only executes when
/// `LocalPlayerRole` is present). This system fills the gap on a dedicated
/// server where `repellent_update` never runs.
fn repellent_ghost_delta_tick(
    mut q_ghost: Query<&mut GhostSprite>,
    difficulty: Res<CurrentDifficulty>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for mut ghost in q_ghost.iter_mut() {
        if ghost.repellent_hits_frame >= 1.0 {
            while ghost.repellent_hits_frame >= 1.0 {
                ghost.repellent_hits += 1;
                ghost.repellent_hits_frame -= 1.0;
                ghost.rage += 0.6 * difficulty.0.ghost_rage_likelihood();
            }
            ghost.repellent_hits_delta = 1.0;
        } else {
            ghost.repellent_hits_frame = (ghost.repellent_hits_frame - dt).max(0.0);
            ghost.repellent_hits_delta -= dt;
            ghost.repellent_hits_delta = ghost
                .repellent_hits_delta
                .clamp(0.0, 1.0)
                .max(ghost.repellent_hits_frame);
        }
        if ghost.repellent_misses_frame >= 1.0 {
            while ghost.repellent_misses_frame >= 1.0 {
                ghost.repellent_misses += 1;
                ghost.repellent_misses_frame -= 1.0;
                ghost.rage += 0.6 * difficulty.0.ghost_rage_likelihood();
            }
            ghost.repellent_misses_delta = 1.0;
        } else {
            ghost.repellent_misses_frame = (ghost.repellent_misses_frame - dt).max(0.0);
            ghost.repellent_misses_delta -= dt;
            ghost.repellent_misses_delta = ghost
                .repellent_misses_delta
                .clamp(0.0, 1.0)
                .max(ghost.repellent_misses_frame);
        }
    }
}
