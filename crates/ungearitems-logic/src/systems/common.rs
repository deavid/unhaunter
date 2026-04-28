use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use rand::prelude::*;
use uncommon_app_core::random_seed;
use ungear_core::components::core::{Battery, Electronic};
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use ungear_core::types::gear::kind::GearKind;
use ungearitems_core::components::repellentflask::RepellentFlask;
use ungearitems_core::events::RepellentUsedEvent;
use ungearitems_core::events::RequestCraftRepellent;
use unghost_core::resources::haunt_state::HauntState;
use uninteraction_core::interaction::Toggleable;
use unmetrics_core::metrics::SendMetric;
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unspatial_core::position::Position;

pub(crate) fn system_electronic_interference(
    haunt_state: Res<HauntState>,
    mut q_electronic: Query<(&Position, &mut Electronic, &Toggleable)>,
    time: Res<Time>,
) {
    let measure = crate::metrics::ELECTRONIC_INTERFERENCE.time_measure();
    let mut rng = random_seed::rng();
    let dt = time.delta_secs();

    for (pos, mut electronic, toggle) in q_electronic.iter_mut() {
        if electronic.glitch_timer > 0.0 {
            electronic.glitch_timer -= dt;
        }

        if let Some(ghost_pos) = &haunt_state.ghost_warning_position {
            let distance2 = pos.distance2(ghost_pos);
            if haunt_state.ghost_warning_intensity > 0.0001 && toggle.is_on {
                let effect_strength = haunt_state.ghost_warning_intensity
                    * (100.0 / distance2).min(1.0)
                    * electronic.sensitivity;

                electronic.glitch_intensity = effect_strength;

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

pub(crate) fn update_repellentflask_skeleton(
    mut q_repellent: Query<(Entity, &mut RepellentFlask), With<LocallyOwned>>,
    q_triggered: Query<&uninteraction_core::interaction::Triggered>,
    mut commands: Commands,
    mut ev_repellent: MessageWriter<RepellentUsedEvent>,
) {
    for (entity, mut repellent) in q_repellent.iter_mut() {
        if q_triggered.get(entity).is_ok()
            && !repellent.active
            && repellent.qty > 0
            && repellent.liquid_content.is_some()
        {
            repellent.active = true;
            commands
                .entity(entity)
                .remove::<uninteraction_core::interaction::Triggered>();
        }

        if repellent.active {
            let mut rng = random_seed::rng();
            if rng.random_range(0.0..1.0) <= 0.5 {
                if repellent.qty == RepellentFlask::MAX_QTY {
                    ev_repellent.write(RepellentUsedEvent);
                }
                repellent.qty -= 1;
                if repellent.qty <= 0 {
                    repellent.qty = 0;
                    repellent.active = false;
                }
            }
        }
    }
}

pub(crate) fn system_battery_drain(
    time: Res<Time>,
    mut q_battery: Query<(&mut Battery, &mut Toggleable)>,
) {
    let measure = crate::metrics::BATTERY_DRAIN.time_measure();
    let dt = time.delta_secs();

    for (mut battery, mut toggle) in q_battery.iter_mut() {
        if toggle.is_on {
            battery.level -= battery.drain_rate * dt;
            if battery.level <= 0.0 {
                battery.level = 0.0;
                toggle.is_on = false;
            }
        }
    }
    measure.end_ms();
}

pub(crate) fn handle_craft_repellent_request(
    mut ev_craft: MessageReader<RequestCraftRepellent>,
    mut q_gear: Query<(&mut PlayerGear, &Owner)>,
    gear_registry: Res<GearSpawnerRegistry>,
    q_gearkind: Query<&GearKind>,
    mut commands: Commands,
) {
    let events: Vec<RequestCraftRepellent> = ev_craft.read().cloned().collect();
    if events.is_empty() {
        return;
    }

    debug!(
        "REPELLENT: authority craft handler received {} request(s)",
        events.len()
    );

    for ev in events {
        let ghost_type = ev.ghost_type;

        let Ok((mut p_gear, player_owner)) = q_gear.get_mut(ev.player_entity) else {
            warn!(
                "REPELLENT: craft handler received request for entity {:?} but could not find its PlayerGear",
                ev.player_entity
            );
            continue;
        };

        debug!(
            "REPELLENT: processing craft request ghost_type={:?} for player {:?} with gear state left={:?} right={:?} inventory={:?}",
            ghost_type, ev.player_entity, p_gear.left_hand, p_gear.right_hand, p_gear.inventory
        );

        // 1) Find an existing RepellentFlask in any slot.
        enum FlaskSlot {
            Left,
            Right,
            Inv(usize),
        }
        let mut old_flask: Option<(Entity, FlaskSlot)> = None;

        if let Some(e) = p_gear.right_hand
            && let Ok(kind) = q_gearkind.get(e)
            && *kind == GearKind::RepellentFlask
        {
            old_flask = Some((e, FlaskSlot::Right));
        }
        if old_flask.is_none()
            && let Some(e) = p_gear.left_hand
            && let Ok(kind) = q_gearkind.get(e)
            && *kind == GearKind::RepellentFlask
        {
            old_flask = Some((e, FlaskSlot::Left));
        }
        if old_flask.is_none() {
            for (idx, &e) in p_gear.inventory.iter().enumerate() {
                if let Ok(kind) = q_gearkind.get(e)
                    && *kind == GearKind::RepellentFlask
                {
                    old_flask = Some((e, FlaskSlot::Inv(idx)));
                    break;
                }
            }
        }

        let new_entity = gear_registry.spawn(&mut commands, GearKind::RepellentFlask);
        let rng_val = random_seed::heavy_rng_seed();
        let net_id = NetworkId(rng_val.max(1000));

        let mut ec = commands.entity(new_entity);
        ec.insert((
            net_id,
            Replicated,
            Owner(player_owner.0),
            unreplicon_core::components::SimulationAuthorized,
        ));

        if player_owner.0 == OwnerId::Server {
            ec.insert(LocallyOwned);
        }

        ec.insert(RepellentFlask {
            liquid_content: Some(ghost_type),
            qty: RepellentFlask::MAX_QTY,
            active: false,
        });

        if let Some((old_e, slot)) = old_flask {
            match slot {
                FlaskSlot::Left => p_gear.left_hand = Some(new_entity),
                FlaskSlot::Right => p_gear.right_hand = Some(new_entity),
                FlaskSlot::Inv(idx) => p_gear.inventory[idx] = new_entity,
            }
            commands.entity(old_e).despawn();
        } else {
            if let Some(old_rh) = p_gear.right_hand.take() {
                if p_gear.inventory.len() < 2 {
                    p_gear.inventory.push(old_rh);
                } else {
                    commands.entity(old_rh).despawn();
                }
            }
            p_gear.right_hand = Some(new_entity);
        }

        debug!(
            "REPELLENT: finished craft request ghost_type={:?} resulting gear state left={:?} right={:?} inventory={:?}",
            ghost_type, p_gear.left_hand, p_gear.right_hand, p_gear.inventory
        );
    }
}
