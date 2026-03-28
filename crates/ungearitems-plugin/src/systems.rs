use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use rand::prelude::*;
use unaudiospatial_core::emitter::AudioEmitter;
use uncommon_app_core::random_seed;
use ungear_core::components::core::{Battery, Electronic};
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use ungear_core::types::gear::kind::GearKind;
use ungearitems_core::components::repellentflask::RepellentFlask;
use ungearitems_core::events::RequestCraftRepellent;
use unghost_core::resources::haunt_state::HauntState;
use uninteraction_core::interaction::Toggleable;
use unmetrics_core::metrics::SendMetric;
use unplayer_core::components::MainPlayer;
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unspatial_core::position::Position;

use crate::metrics;

pub(crate) fn system_electronic_interference(
    gs_audio: AudioEmitter,
    haunt_state: Res<HauntState>,
    mut q_electronic: Query<(&Position, &mut Electronic, &Toggleable)>,
) {
    let measure = metrics::ELECTRONIC_INTERFERENCE.time_measure();
    let mut rng = random_seed::rng();
    let dt = gs_audio.time.delta_secs();

    for (pos, mut electronic, toggle) in q_electronic.iter_mut() {
        // Decrement glitch timer if active
        if electronic.glitch_timer > 0.0 {
            electronic.glitch_timer -= dt;
        }

        // Apply EMI if warning is active and item is on
        if let Some(ghost_pos) = &haunt_state.ghost_warning_position {
            let distance2 = pos.distance2(ghost_pos);
            if haunt_state.ghost_warning_intensity > 0.0001 && toggle.is_on {
                // Scale effect by distance and warning level
                let effect_strength = haunt_state.ghost_warning_intensity
                    * (100.0 / distance2).min(1.0)
                    * electronic.sensitivity;

                electronic.glitch_intensity = effect_strength;

                // Random glitches
                if rng.random_range(0.0..1.0) < effect_strength.powi(2) {
                    electronic.glitch_timer = rng.random_range(0.2..0.6);
                }
            } else {
                electronic.glitch_intensity = 0.0;
            }
        } else {
            electronic.glitch_intensity = 0.0;
        }
    }

    measure.end_ms();
}

pub(crate) fn system_battery_drain(
    time: Res<Time>,
    mut q_battery: Query<(&mut Battery, &mut Toggleable)>,
) {
    let measure = metrics::BATTERY_DRAIN.time_measure();
    let dt = time.delta_secs();

    for (mut battery, mut toggle) in q_battery.iter_mut() {
        if toggle.is_on {
            battery.level -= battery.drain_rate * dt;
            if battery.level <= 0.0 {
                battery.level = 0.0;
                toggle.is_on = false; // Auto-shutdown
            }
        }
    }

    measure.end_ms();
}

/// Authority-side handler for `RequestCraftRepellent` emitted by `untruck-plugin`.
///
/// Finds or creates a `RepellentFlask` in the local player's gear and fills it with the
/// requested ghost type's repellent. Mirrors the logic previously in
/// `untruck-plugin::craft_repellent`.
pub(crate) fn handle_craft_repellent_request(
    mut ev_craft: MessageReader<RequestCraftRepellent>,
    mut q_gear: Query<&mut PlayerGear, With<MainPlayer>>,
    gear_registry: Res<GearSpawnerRegistry>,
    mut q_repellent: Query<&mut RepellentFlask>,
    q_gearkind: Query<&GearKind>,
    mut commands: Commands,
) {
    let Ok(mut p_gear) = q_gear.single_mut() else {
        // No local MainPlayer: dedicated server mode or pre-spawn. Consume and discard.
        for _ in ev_craft.read() {}
        return;
    };

    for ev in ev_craft.read() {
        let ghost_type = ev.ghost_type;

        // 1) Find an existing RepellentFlask in any slot.
        let mut flask_entity: Option<Entity> = None;

        if let Some(e) = p_gear.right_hand
            && let Ok(kind) = q_gearkind.get(e)
            && *kind == GearKind::RepellentFlask
        {
            flask_entity = Some(e);
        }

        if flask_entity.is_none()
            && let Some(e) = p_gear.left_hand
            && let Ok(kind) = q_gearkind.get(e)
            && *kind == GearKind::RepellentFlask
        {
            flask_entity = Some(e);
        }

        if flask_entity.is_none() {
            for &e in &p_gear.inventory {
                if let Ok(kind) = q_gearkind.get(e)
                    && *kind == GearKind::RepellentFlask
                {
                    flask_entity = Some(e);
                    break;
                }
            }
        }

        // 2) If none found, spawn a new flask and place it in the right hand.
        let is_new = flask_entity.is_none();
        if is_new {
            let entity = gear_registry.spawn(&mut commands, GearKind::RepellentFlask);
            let rng_val = random_seed::heavy_rng_seed();
            let net_id = NetworkId(rng_val.max(1000));
            commands.entity(entity).insert((
                net_id,
                Replicated,
                Owner(OwnerId::Server),
                LocallyOwned,
            ));

            // Put in right hand, moving old item to inventory or despawning if full.
            if let Some(old_rh) = p_gear.right_hand.take() {
                if p_gear.inventory.len() < 2 {
                    p_gear.inventory.push(old_rh);
                } else {
                    commands.entity(old_rh).despawn();
                }
            }
            p_gear.right_hand = Some(entity);
            flask_entity = Some(entity);
        }

        // 3) Fill the flask.
        let Some(entity) = flask_entity else {
            error!(
                "handle_craft_repellent_request: flask_entity is None after search and spawn — should be unreachable"
            );
            continue;
        };

        if is_new {
            // Insert the desired state directly; the entity is too new for the query.
            commands.entity(entity).insert(RepellentFlask {
                liquid_content: Some(ghost_type),
                qty: RepellentFlask::MAX_QTY,
                active: false,
            });
        } else if let Ok(mut flask) = q_repellent.get_mut(entity) {
            flask.liquid_content = Some(ghost_type);
            flask.qty = RepellentFlask::MAX_QTY;
            flask.active = false;
        } else {
            error!(
                "handle_craft_repellent_request: Failed to get RepellentFlask on entity {:?}",
                entity
            );
        }
    }
}
