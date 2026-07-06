use bevy::prelude::*;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unfog_core::miasma::MiasmaGrid;
use unghost_core::resources::signals::GhostHuntSignals;
use unplayer_core::components::{MainPlayer, PlayerSpectating, PlayerSprite};
use unreplicon_core::ownership::LocallyOwned;
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;
use unvitals_core::components::{PlayerVitals, Stamina};
use unvitals_core::events::PlayerDiedEvent;

/// Update asphyxia levels based on miasma pressure with three cascading time constants.
/// Applies cbrt curve first, then flows through: 2s immediate → 5s acute → 60s chronic.
pub(crate) fn update_asphyxia_from_miasma(
    time: Res<Time>,
    mut qp: Query<
        (&mut PlayerVitals, &Position),
        (
            With<LocallyOwned>,
            Without<InTruck>,
            Without<PlayerSpectating>,
        ),
    >,
    miasma: Option<Res<MiasmaGrid>>,
) {
    let dt = time.delta_secs();

    for (mut vitals, pos) in &mut qp {
        let miasma_pressure = if let Some(miasma) = miasma.as_ref() {
            let bpos = pos.to_board_position();
            miasma
                .pressure_field
                .get(bpos.ndidx())
                .copied()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Apply cbrt curve early to compress the scale
        // This ensures 10k doesn't have outsized weight compared to small values
        let cbrt_pressure = f32::cbrt(miasma_pressure);

        // Tier 1: Immediate (symmetric).
        // Always 2-second response time, both buildup and decay.
        const IMMEDIATE_RATE: f32 = 0.5;
        vitals.asphyxia_immediate = vitals.asphyxia_immediate * (1.0 - IMMEDIATE_RATE * dt)
            + cbrt_pressure * (IMMEDIATE_RATE * dt);

        // Tier 2: Acute (asymmetric: 5s attack / 2s decay).
        let acute_rate = if vitals.asphyxia_immediate > vitals.asphyxia_acute {
            0.2 // 5s attack (buildup)
        } else {
            0.5 // 2s decay (recovers faster in clean air)
        };
        vitals.asphyxia_acute = vitals.asphyxia_acute * (1.0 - acute_rate * dt)
            + vitals.asphyxia_immediate * (acute_rate * dt);

        // Tier 3: Chronic (highly asymmetric: 60s attack / 15s decay).
        // Sustained exposure builds chronic fatigue over 60 seconds.
        // But catching breath in clean air clears the fatigue in ~15 seconds.
        let chronic_rate = if vitals.asphyxia_acute > vitals.asphyxia_chronic {
            1.0 / 60.0 // 60s attack (slow buildup of fatigue)
        } else {
            1.0 / 10.0 // 10s decay (faster recovery from fatigue)
        };
        vitals.asphyxia_chronic = vitals.asphyxia_chronic * (1.0 - chronic_rate * dt)
            + vitals.asphyxia_acute * (chronic_rate * dt);
    }
}

pub(crate) fn debug_log_asphyxia(
    mut interval: Local<f32>,
    time: Res<Time>,
    qp: Query<(&PlayerVitals, &PlayerSprite), (With<LocallyOwned>, With<MainPlayer>)>,
) {
    let dt = time.delta_secs();
    *interval += dt;

    if *interval >= 30.0 {
        *interval = 0.0;
        for (vitals, player) in &qp {
            // Calculate movement penalty from asphyxia
            let effective_asphyxia = (vitals.asphyxia_acute + vitals.asphyxia_chronic) / 2.0;
            let asphyxia_speed_mult = 1.0 / (1.0 + effective_asphyxia / 10.0);
            let movement_penalty_pct = (1.0 - asphyxia_speed_mult) * 100.0;

            debug!(
                "Player {:?} asphyxia — immediate: {:.2}, acute: {:.2}, chronic: {:.2} | movement penalty: {:.1}%",
                player.id,
                vitals.asphyxia_immediate,
                vitals.asphyxia_acute,
                vitals.asphyxia_chronic,
                movement_penalty_pct
            );
        }
    }
}

pub(crate) fn regenerate_health_over_time(
    time: Res<Time>,
    mut qp: Query<
        (&mut PlayerVitals, &Position),
        (
            With<LocallyOwned>,
            Without<InTruck>,
            Without<PlayerSpectating>,
        ),
    >,
    difficulty: Res<CurrentDifficulty>,
    miasma: Option<Res<MiasmaGrid>>,
) {
    let dt = time.delta_secs();
    for (mut ps, pos) in &mut qp {
        let miasma_pressure = if let Some(miasma) = miasma.as_ref() {
            let bpos = pos.to_board_position();
            miasma
                .pressure_field
                .get(bpos.ndidx())
                .copied()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Prevent healing in very high miasma concentrations.
        // Penalty starts at 1000 and completely stops healing at 10000.
        let healing_penalty = ((miasma_pressure - 1000.0) / 9000.0).clamp(0.0, 1.0);
        let healing_mult = 1.0 - healing_penalty;

        if ps.health < 100.0 && ps.health > 0.0 {
            ps.health += (0.1 * dt + (1.0 - ps.health / 100.0) * dt * 10.0)
                * difficulty.0.health_recovery_rate()
                * healing_mult;
        }
        if ps.health > 100.0 {
            ps.health = 100.0;
        }
    }
}

pub(crate) fn recover_sanity_in_truck(
    time: Res<Time>,
    mut qp: Query<&mut PlayerVitals, (With<MainPlayer>, With<InTruck>, Without<PlayerSpectating>)>,
    difficulty: Res<CurrentDifficulty>,
) {
    // Players recover sanity while in the truck.
    let dt = time.delta_secs();
    for mut ps in &mut qp {
        // --- Gradual Health Recovery --- Health points recovered per second
        const HEALTH_RECOVERY_RATE: f32 = 2.0;
        if ps.health < 100.0 {
            ps.health += HEALTH_RECOVERY_RATE * dt;

            // Clamp health to a maximum of 100%
            ps.health = ps.health.min(100.0);
        }
        if ps.sanity < difficulty.0.max_recoverable_sanity() {
            ps.crazyness /= 1.07_f32.powf(dt);
        } else {
            ps.crazyness /= 1.005_f32.powf(dt);
        }
        ps.sanity = super::sanity::calculate_sanity(ps.crazyness);
    }
}

pub(crate) fn scale_stamina_rates_by_health(
    mut players: Query<(&PlayerVitals, &mut Stamina)>,
    difficulty: Res<CurrentDifficulty>,
) {
    for (player_vitals, mut stamina) in players.iter_mut() {
        // Adjust stamina parameters based on health
        let health_percentage = player_vitals.health / 100.0;

        // When health is low, stamina depletes faster and recovers slower
        if health_percentage < 0.3 {
            stamina.depletion_rate = 1.2 * difficulty.0.health_recovery_rate(); // Depletes 50% faster when health is critical
            stamina.recovery_rate = 0.15 * difficulty.0.health_recovery_rate(); // Recovers 50% slower when health is critical
        } else if health_percentage < 0.6 {
            stamina.depletion_rate = 1.0 * difficulty.0.health_recovery_rate(); // Depletes 25% faster when health is low
            stamina.recovery_rate = 0.2 * difficulty.0.health_recovery_rate(); // Recovers 33% slower when health is low
        } else {
            // Reset to default rates based on difficulty
            stamina.depletion_rate = 0.8 * difficulty.0.health_recovery_rate();
            stamina.recovery_rate = 0.3 * difficulty.0.health_recovery_rate();
        }
    }
}

pub(crate) fn transition_to_spectator_on_death(
    mut commands: Commands,
    mut player_query: Query<(Entity, &PlayerSprite, &PlayerVitals), Without<PlayerSpectating>>,
    mut ev_death: MessageWriter<PlayerDiedEvent>,
) {
    for (entity, player, vitals) in player_query.iter_mut() {
        if vitals.health <= 0.0 {
            info!(
                "Player {:?} ({:?}) died! Entering spectate mode.",
                entity, player.id
            );
            commands.entity(entity).insert(PlayerSpectating);

            ev_death.write(PlayerDiedEvent {
                id: player.network_id,
            });
        }
    }
}

pub(crate) fn debug_kill_spectator(
    keyboard_input: Option<Res<ButtonInput<KeyCode>>>,
    mut player_query: Query<&mut PlayerVitals, With<MainPlayer>>,
) {
    let Some(keyboard_input) = keyboard_input else {
        return;
    };
    let shift =
        keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);
    let ctrl = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight);

    if shift && ctrl && keyboard_input.just_pressed(KeyCode::KeyK) {
        for mut vitals in player_query.iter_mut() {
            vitals.health = -10.0;
        }
    }
}

pub(crate) fn apply_miasma_hazard_damage(
    mut damage_ev: MessageReader<unfog_core::messages::MiasmaTakeDamageMessage>,
    mut q_vitals: Query<&mut PlayerVitals>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for ev in damage_ev.read() {
        if let Ok(mut vitals) = q_vitals.get_mut(ev.target_entity) {
            vitals.health -= ev.damage * dt;
        }
    }
}

pub(crate) fn apply_ghost_proximity_damage(
    mut q_local_player: Query<
        (&Position, &mut PlayerVitals),
        (
            With<LocallyOwned>,
            Without<PlayerSpectating>,
            Without<InTruck>,
        ),
    >,
    hunt_signals: Res<GhostHuntSignals>,
    time: Res<Time>,
    difficulty: Res<CurrentDifficulty>,
    mut hunt_start: Local<f32>,
) {
    let dt = time.delta_secs();

    let Ok((player_pos, mut vitals)) = q_local_player.single_mut() else {
        return;
    };

    if !hunt_signals.any_hunting {
        *hunt_start = 0.0;
    }

    for pressure in hunt_signals.pressures.iter() {
        if pressure.is_warping {
            *hunt_start = time.elapsed_secs(); // Reset timer silently
            continue; // Skip damage entirely while mid-dash
        }

        if *hunt_start == 0.0 {
            *hunt_start = time.elapsed_secs();
        }
        let ghost_strength = (time.elapsed_secs() - *hunt_start).clamp(0.0, 2.0);

        let dist2 = player_pos.weighted_distance_squared(&pressure.position) + 2.0;

        let dmg = dist2.recip() * difficulty.0.health_drain_rate();
        let damage_to_apply =
            dmg * dt * 30.0 * ghost_strength / (1.0 + pressure.calm_time_secs / 5.0);
        vitals.health -= damage_to_apply;
    }
}
