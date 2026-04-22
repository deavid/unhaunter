use bevy::prelude::*;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unghost_core::resources::signals::GhostHuntSignals;
use unplayer_core::components::{MainPlayer, PlayerSpectating, PlayerSprite};
use unreplicon_core::ownership::LocallyOwned;
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;
use unvitals_core::components::{PlayerVitals, Stamina};
use unvitals_core::events::PlayerDiedEvent;

pub(crate) fn regenerate_health_over_time(
    time: Res<Time>,
    mut qp: Query<
        &mut PlayerVitals,
        (
            With<LocallyOwned>,
            Without<InTruck>,
            Without<PlayerSpectating>,
        ),
    >,
    difficulty: Res<CurrentDifficulty>,
) {
    let dt = time.delta_secs();
    for mut ps in &mut qp {
        if ps.health < 100.0 && ps.health > 0.0 {
            ps.health += (0.1 * dt + (1.0 - ps.health / 100.0) * dt * 10.0)
                * difficulty.0.health_recovery_rate();
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
