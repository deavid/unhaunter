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
use unplayer_core::components::MainPlayer;
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
    mut q_gear: Query<&mut PlayerGear, With<MainPlayer>>,
    q_player_gear_entities: Query<(Entity, Has<MainPlayer>, Option<&Owner>), With<PlayerGear>>,
    gear_registry: Res<GearSpawnerRegistry>,
    mut q_repellent: Query<&mut RepellentFlask>,
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

    let Ok(mut p_gear) = q_gear.single_mut() else {
        let candidates: Vec<(Entity, bool, Option<OwnerId>)> = q_player_gear_entities
            .iter()
            .map(|(entity, is_main_player, owner)| {
                (entity, is_main_player, owner.map(|owner| owner.0))
            })
            .collect();
        warn!(
            "REPELLENT: craft handler received {} request(s) but could not resolve a unique MainPlayer PlayerGear; candidates={:?}",
            events.len(),
            candidates
        );
        return;
    };

    for ev in events {
        let ghost_type = ev.ghost_type;

        debug!(
            "REPELLENT: processing craft request ghost_type={:?} with gear state left={:?} right={:?} inventory={:?}",
            ghost_type, p_gear.left_hand, p_gear.right_hand, p_gear.inventory
        );

        // 1) Find an existing RepellentFlask in any slot.
        let mut flask_entity: Option<Entity> = None;

        if let Some(e) = p_gear.right_hand {
            match q_gearkind.get(e) {
                Ok(kind) if *kind == GearKind::RepellentFlask => {
                    debug!(
                        "REPELLENT: found existing flask in right hand entity {:?}",
                        e
                    );
                    flask_entity = Some(e);
                }
                Ok(kind) => {
                    debug!(
                        "REPELLENT: right hand entity {:?} is {:?}, not RepellentFlask",
                        e, kind
                    );
                }
                Err(err) => {
                    warn!(
                        "REPELLENT: failed to read GearKind for right hand entity {:?}: {}",
                        e, err
                    );
                }
            }
        }

        if flask_entity.is_none()
            && let Some(e) = p_gear.left_hand
        {
            match q_gearkind.get(e) {
                Ok(kind) if *kind == GearKind::RepellentFlask => {
                    debug!(
                        "REPELLENT: found existing flask in left hand entity {:?}",
                        e
                    );
                    flask_entity = Some(e);
                }
                Ok(kind) => {
                    debug!(
                        "REPELLENT: left hand entity {:?} is {:?}, not RepellentFlask",
                        e, kind
                    );
                }
                Err(err) => {
                    warn!(
                        "REPELLENT: failed to read GearKind for left hand entity {:?}: {}",
                        e, err
                    );
                }
            }
        }

        if flask_entity.is_none() {
            for &e in &p_gear.inventory {
                match q_gearkind.get(e) {
                    Ok(kind) if *kind == GearKind::RepellentFlask => {
                        debug!(
                            "REPELLENT: found existing flask in inventory entity {:?}",
                            e
                        );
                        flask_entity = Some(e);
                        break;
                    }
                    Ok(kind) => {
                        debug!(
                            "REPELLENT: inventory entity {:?} is {:?}, not RepellentFlask",
                            e, kind
                        );
                    }
                    Err(err) => {
                        warn!(
                            "REPELLENT: failed to read GearKind for inventory entity {:?}: {}",
                            e, err
                        );
                    }
                }
            }
        }

        // 2) If none found, spawn a new flask and place it in the right hand.
        let is_new = flask_entity.is_none();
        if is_new {
            let entity = gear_registry.spawn(&mut commands, GearKind::RepellentFlask);
            let rng_val = random_seed::heavy_rng_seed();
            let net_id = NetworkId(rng_val.max(1000));

            debug!(
                "REPELLENT: spawning new RepellentFlask entity {:?} net_id={:?}",
                entity, net_id
            );

            commands.entity(entity).insert((
                net_id,
                Replicated,
                Owner(OwnerId::Server),
                LocallyOwned,
            ));

            // Put in right hand, moving old item to inventory or despawning if full.
            if let Some(old_rh) = p_gear.right_hand.take() {
                if p_gear.inventory.len() < 2 {
                    debug!(
                        "REPELLENT: moving existing right hand entity {:?} into inventory to make room for new flask",
                        old_rh
                    );
                    p_gear.inventory.push(old_rh);
                } else {
                    warn!(
                        "REPELLENT: inventory full while crafting; despawning previous right hand entity {:?} to make room for new flask",
                        old_rh
                    );
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
            debug!(
                "REPELLENT: inserting freshly crafted flask state on new entity {:?} ghost_type={:?}",
                entity, ghost_type
            );
            commands.entity(entity).insert(RepellentFlask {
                liquid_content: Some(ghost_type),
                qty: RepellentFlask::MAX_QTY,
                active: false,
            });
        } else if let Ok(mut flask) = q_repellent.get_mut(entity) {
            debug!(
                "REPELLENT: refilling existing flask entity {:?} with ghost_type={:?}",
                entity, ghost_type
            );
            flask.liquid_content = Some(ghost_type);
            flask.qty = RepellentFlask::MAX_QTY;
            flask.active = false;
        } else {
            error!(
                "handle_craft_repellent_request: Failed to get RepellentFlask on entity {:?}",
                entity
            );
        }

        debug!(
            "REPELLENT: finished craft request ghost_type={:?} resulting gear state left={:?} right={:?} inventory={:?}",
            ghost_type, p_gear.left_hand, p_gear.right_hand, p_gear.inventory
        );
    }
}
