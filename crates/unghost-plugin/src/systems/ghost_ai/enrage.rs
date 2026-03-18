use super::movement::spawn_salty_trace;
use super::roar::{RoarDecision, RoarReason, RoarType, execute_roar_decision};
use crate::components::fade_out::FadeOut;
use crate::metrics::GHOST_ENRAGE;
use bevy::prelude::*;
use rand::RngExt;
use unaudiobg_core::events::AmbientSoundMuteEvent;
use unboard_core::resources::board_topology::BoardCollisionField;
use unboard_core::resources::roomdb::RoomTopology;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unfoundation_core::random_seed;
use unfoundation_core::utils::mean::MeanValue;
use unfoundation_core::utils::time::PrintingTimer;
use unghost_core::components::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unmetrics_core::metrics::SendMetric;
use unplayer_core::components::{
    Hiding, PlayerDisconnected, PlayerInactive, PlayerSpectating, PlayerSprite,
};
use unsound_core::emitter::SoundEmitter;
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;

/// Enables/disables debug logs for hunting behavior.
const DEBUG_HUNTS: bool = true;

/// Result of rage calculation
#[derive(Debug)]
pub(crate) struct RageUpdateResult {
    pub rage_limit: f32,
    pub should_trigger_hunt: bool,
}

/// Manages the ghost's rage level, hunting behavior, and player interactions
/// during a hunt.
///
/// This system updates the ghost's rage based on player proximity, sanity, and
/// sound levels. It triggers hunts when rage exceeds a threshold.
/// Player health damage during hunts is handled client-side by
/// `client_ghost_aura_damage` in `unplayer-plugin`.
pub(crate) fn ghost_enrage(
    mut timer: Local<PrintingTimer>,
    mut avg_angry: Local<MeanValue>,
    mut qg: Query<(&mut GhostSprite, &Position, &GhostBehaviorDynamics), Without<FadeOut>>,
    q_player: Query<
        (&PlayerSprite, &Position, Option<&Hiding>),
        (
            Without<PlayerSpectating>,
            Without<PlayerDisconnected>,
            Without<PlayerInactive>,
            Without<InTruck>,
        ),
    >,
    mut gs_audio: SoundEmitter,
    mut commands: Commands,
    board_collision: Res<BoardCollisionField>,
    mut last_roar: Local<f32>,
    difficulty: Res<CurrentDifficulty>,
    room_topology: Res<RoomTopology>,
    mut ev_ambient_mute: Option<MessageWriter<AmbientSoundMuteEvent>>,
) {
    let measure = GHOST_ENRAGE.time_measure();

    timer.tick(gs_audio.time.delta());
    let dt = gs_audio.time.delta_secs();
    *last_roar += dt;

    for (mut ghost, ghost_position, dynamics) in qg.iter_mut() {
        // 1. Update basic timers
        update_ghost_timers_simple(&mut ghost, dt, &gs_audio.time);

        // 2. Handle salty trace spawning
        handle_salty_trace_spawning_simple(
            &mut ghost,
            ghost_position,
            &mut commands,
            &gs_audio.asset_server,
            &board_collision,
        );

        // 3. Calculate minimum player distance for this ghost
        let min_player_dist = calculate_min_player_distance(ghost_position, &q_player);

        // 4. Apply distance-based effects
        apply_distance_based_effects(&mut ghost, min_player_dist, dt);

        // 5. Handle hunting phase
        if ghost.hunt_target {
            let hunt_result = handle_hunting_phase(&mut ghost, dt);

            if hunt_result.should_roar {
                let roar_decision = RoarDecision {
                    roar_type: hunt_result.roar_type,
                    should_play_now: *last_roar > 3.0,
                    reason: RoarReason::HuntingInProgress,
                    time_override: None,
                };

                execute_roar_decision(
                    &roar_decision,
                    &mut last_roar,
                    &mut gs_audio,
                    ghost_position,
                );
            }
            continue;
        }

        // 6. Handle pre-warning and warning phases
        let warning_result =
            handle_warning_phases(&mut ghost, dt, &gs_audio.time, &mut ev_ambient_mute);

        // 7. Calculate rage
        let rage_result = calculate_rage_update(
            &mut ghost,
            ghost_position,
            &q_player,
            dynamics,
            &mut avg_angry,
            &difficulty,
            &room_topology,
            dt,
        );

        // 8. Check for hunt trigger
        if should_trigger_hunt(&ghost, &rage_result) {
            trigger_hunt_start(&mut ghost, &rage_result, &difficulty, &mut ev_ambient_mute);
        }

        // 9. Determine roar behavior
        let roar_decision =
            determine_roar_decision(&ghost, &rage_result, &warning_result, *last_roar, dt);

        // 9.5. Handle rage limit multiplier decay (for rage-based roars)
        if matches!(roar_decision.reason, RoarReason::RageBuildup)
            && ghost.rage_limit_multiplier > 1.0
        {
            ghost.rage_limit_multiplier /= 1.01_f32.powf(dt);
        }

        #[allow(clippy::explicit_auto_deref)]
        execute_roar_decision(
            &roar_decision,
            &mut *last_roar,
            &mut gs_audio,
            ghost_position,
        );

        // 10. Debug logging
        if timer.just_finished() && DEBUG_HUNTS {
            debug_log_ghost_state(&ghost, &rage_result, &roar_decision);
        }
    }

    measure.end_ms();
}

// Helper functions for simplified ghost behavior processing

/// Updates basic ghost timers
fn update_ghost_timers_simple(ghost: &mut GhostSprite, dt: f32, time: &Res<Time>) {
    // Update calm time
    if ghost.calm_time_secs > 0.0 {
        ghost.calm_time_secs -= dt.min(ghost.calm_time_secs);
    }

    // Update salty effect timers if active
    if !ghost.salty_effect_timer.is_finished() && ghost.hunting <= 0.1 {
        ghost.salty_effect_timer.tick(time.delta());
        ghost.salty_trace_spawn_timer.tick(time.delta());
    }
}

/// Spawns salty traces when conditions are met
fn handle_salty_trace_spawning_simple(
    ghost: &mut GhostSprite,
    ghost_position: &Position,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    board_collision: &BoardCollisionField,
) {
    if !ghost.salty_effect_timer.is_finished()
        && ghost.hunting <= 0.1
        && ghost.salty_trace_spawn_timer.just_finished()
    {
        if random_seed::rng().random_bool(0.5) {
            // Find valid floor tile
            let ghost_board_position = ghost_position.to_board_position();
            let mut valid_tile = None;
            for nearby_tile in ghost_board_position.iter_xy_neighbors_nosize(1) {
                let collision_data = board_collision.0[nearby_tile.ndidx()];
                if collision_data.player_free {
                    valid_tile = Some(nearby_tile);
                    break;
                }
            }

            if let Some(tile_position) = valid_tile {
                spawn_salty_trace(commands, asset_server, tile_position);
            }
        }
        ghost.salty_trace_spawn_timer.reset();
    }
}

/// Calculate minimum player distance to ghost
/// Calculate minimum distance to any alive player
fn calculate_min_player_distance(
    ghost_position: &Position,
    q_player: &Query<
        (&PlayerSprite, &Position, Option<&Hiding>),
        (
            Without<PlayerSpectating>,
            Without<PlayerDisconnected>,
            Without<PlayerInactive>,
            Without<InTruck>,
        ),
    >,
) -> f32 {
    q_player
        .iter()
        .filter(|(p, _, _)| p.health > 0.0)
        .map(|(_, pos, _)| calculate_weighted_distance(ghost_position, pos))
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(1000.0)
        .clamp(1.0, 1000.0)
}

/// Apply distance-based effects on ghost behavior
fn apply_distance_based_effects(ghost: &mut GhostSprite, min_player_dist: f32, dt: f32) {
    // Reduce ghost rage as player is further away
    ghost.rage -= dt * min_player_dist.sqrt() / 10.0;
    if !ghost.hunt_target {
        // Reduce ghost hunting when player is away
        ghost.hunting -= dt * min_player_dist.sqrt() / 3.0;
    }
}

/// Result of hunting phase processing
#[derive(Debug)]
pub(crate) struct HuntingResult {
    pub should_roar: bool,
    pub roar_type: RoarType,
}

/// Handle the hunting phase of ghost behavior
pub(crate) fn handle_hunting_phase(ghost: &mut GhostSprite, dt: f32) -> HuntingResult {
    // Reset warning states during hunting
    ghost.hunt_warning_active = false;
    ghost.hunt_warning_intensity = 1.0;
    ghost.hunt_warning_timer = 0.0;

    // Determine roar during hunting
    let roar_type = if ghost.hunting > 4.0 {
        RoarType::Full
    } else {
        RoarType::Dim
    };

    // Reduce rage during hunting
    ghost.rage -= dt * 20.0;
    if ghost.rage < 0.0 {
        ghost.rage = 0.0;
    }

    HuntingResult {
        should_roar: true,
        roar_type,
    }
}

/// Result of warning phase processing
#[derive(Debug)]
pub(crate) struct WarningResult {
    pub roar_triggered: bool,
    pub roar_type: RoarType,
    pub hunt_started: bool,
}

/// Handle pre-warning and warning phases
pub(crate) fn handle_warning_phases(
    ghost: &mut GhostSprite,
    dt: f32,
    time: &Res<Time>,
    o_ev_ambient_mute: &mut Option<MessageWriter<AmbientSoundMuteEvent>>,
) -> WarningResult {
    let mut result = WarningResult {
        roar_triggered: false,
        roar_type: RoarType::None,
        hunt_started: false,
    };

    // Pre-warning phase
    if ghost.pre_warning_timer > 0.0 {
        ghost.pre_warning_timer -= dt;
        if ghost.pre_warning_timer <= 0.0 {
            // Pre-warning timer expired, start actual hunt warning with roar
            if !ghost.hunt_warning_active {
                ghost.hunt_warning_active = true;
                ghost.hunt_warning_timer = 5.0;
                ghost.hunt_warning_intensity = 0.0;

                result.roar_triggered = true;
                result.roar_type = RoarType::Full;
            }
        }
    }

    // Warning phase
    if ghost.hunt_warning_active {
        ghost.hunt_warning_timer -= dt;
        ghost.hunt_warning_intensity = 1.0 - (ghost.hunt_warning_timer / 10.0);

        // Send stronger mute event when hunt is about to start (anticipatory)
        if ghost.hunt_warning_timer <= 0.5 && ghost.hunt_warning_timer > 0.5 - dt {
            // Send stronger/faster mute for actual hunt start
            if let Some(ev_ambient_mute) = o_ev_ambient_mute {
                ev_ambient_mute.write(AmbientSoundMuteEvent::default());
            }

            // Trigger hunt after warning period
            ghost.hunt_warning_active = false;
            ghost.hunt_warning_intensity = 1.0; //Max intensity
            ghost.hunt_target = true;
            ghost.hunt_time_secs = time.elapsed_secs();
            info!("Hunting player for {:.1}s", ghost.hunting);

            result.hunt_started = true;
        }
    } else if ghost.hunting < 0.001 {
        ghost.hunt_warning_intensity /= 2.2_f32.powf(dt);
    }

    result
}

/// Calculate rage update and return result information
pub(crate) fn calculate_rage_update(
    ghost: &mut GhostSprite,
    ghost_position: &Position,
    q_player: &Query<
        (&PlayerSprite, &Position, Option<&Hiding>),
        (
            Without<PlayerSpectating>,
            Without<PlayerDisconnected>,
            Without<PlayerInactive>,
            Without<InTruck>,
        ),
    >,
    dynamics: &GhostBehaviorDynamics,
    avg_angry: &mut MeanValue,
    difficulty: &Res<CurrentDifficulty>,
    room_topology: &Res<RoomTopology>,
    dt: f32,
) -> RageUpdateResult {
    // Calculate player-induced rage
    let mut total_angry2 = 0.0;
    let mut player_in_room = false;
    let mut total_inv_sanity = 0.0;

    for (player_sprite, player_pos, _) in q_player.iter() {
        let sanity = player_sprite.sanity;
        let inv_sanity = (120.0 - sanity) / 100.0;

        let dist2 = calculate_weighted_distance_squared(ghost_position, player_pos)
            / difficulty.0.hunt_provocation_radius()
            * (0.01 + sanity)
            + 0.1
            + sanity / 100.0;

        let angry2 = dist2.recip() * 1000000.0 / sanity
            * player_sprite.mean_sound
            * (player_sprite.health / 100.0).clamp(0.0, 1.0);

        total_angry2 +=
            angry2 * inv_sanity + player_sprite.mean_sound.sqrt() * inv_sanity * dt * 3000.1;

        let player_board_position = player_pos.to_board_position();
        if room_topology
            .room_tiles
            .contains_key(&player_board_position)
        {
            player_in_room = true;
            total_inv_sanity += inv_sanity;
        }
    }

    let angry = total_angry2.sqrt();
    let a_f = 1.0 + (avg_angry.avg() * 2.0).powi(2);

    // Apply rage decay
    ghost.rage /= 1.01_f32.powf(dt / a_f);
    ghost.rage -= dt * 2.0 / a_f;
    if ghost.rage < 0.0 {
        ghost.rage = 0.0;
    }

    // Apply rage increases
    if player_in_room {
        ghost.rage += dt * difficulty.0.ghost_rage_likelihood() * 5.2 * total_inv_sanity;
    }
    ghost.rage +=
        angry * dt / 10.0 / (1.0 + ghost.calm_time_secs) * difficulty.0.ghost_rage_likelihood();

    // Update hunting decay
    ghost.hunting -= dt * 0.2 / difficulty.0.ghost_hunt_duration();
    if ghost.hunting < 0.0 {
        ghost.hunting = 0.0;
    }

    avg_angry.push_len(angry, dt);

    // Calculate rage limit
    let rage_limit =
        400.0 * difficulty.0.ghost_rage_likelihood().sqrt() * ghost.rage_limit_multiplier
            / (dynamics.rage_tendency_multiplier + 1.01);
    ghost.rage_limit = rage_limit;

    // Determine if hunt should be triggered
    let should_trigger_hunt = ghost.rage > rage_limit
        && !ghost.hunt_warning_active
        && !ghost.hunt_target
        && ghost.pre_warning_timer <= 0.0;

    RageUpdateResult {
        rage_limit,
        should_trigger_hunt,
    }
}

/// Check if hunt should be triggered
pub(crate) fn should_trigger_hunt(_ghost: &GhostSprite, rage_result: &RageUpdateResult) -> bool {
    rage_result.should_trigger_hunt
}

/// Trigger the start of a hunt
pub(crate) fn trigger_hunt_start(
    ghost: &mut GhostSprite,
    _rage_result: &RageUpdateResult,
    difficulty: &Res<CurrentDifficulty>,
    o_ev_ambient_mute: &mut Option<MessageWriter<AmbientSoundMuteEvent>>,
) {
    // Start Pre-Warning Phase (anticipatory audio muting)
    ghost.pre_warning_timer = 3.0;
    ghost.rage_limit_multiplier *= 1.3;

    let prev_rage = ghost.rage;
    ghost.rage /= 1.0 + difficulty.0.ghost_hunt_cooldown();
    ghost.hunting += prev_rage / 50.0 + 5.0;
    ghost.hunt_warning_active = false;

    // Send anticipatory mute event BEFORE the hunt warning begins
    if let Some(ev_ambient_mute) = o_ev_ambient_mute {
        ev_ambient_mute.write(AmbientSoundMuteEvent::default());
    }
}

/// Determine roar decision based on current state
pub(crate) fn determine_roar_decision(
    ghost: &GhostSprite,
    rage_result: &RageUpdateResult,
    warning_result: &WarningResult,
    last_roar_time: f32,
    _dt: f32,
) -> RoarDecision {
    // Handle warning-triggered roars first
    if warning_result.roar_triggered {
        return RoarDecision {
            roar_type: warning_result.roar_type,
            should_play_now: last_roar_time > 0.2,
            reason: RoarReason::HuntWarningStart,
            time_override: Some(0.2),
        };
    }

    // Check for rage-based roars when not hunting
    if ghost.rage > rage_result.rage_limit / 2.0 && ghost.hunting < 1.0 && last_roar_time > 10.0 {
        // Apply rage limit multiplier decay here instead of in main function
        // Note: we can't modify ghost here, so we'll handle this in the main function
        return RoarDecision {
            roar_type: RoarType::Dim,
            should_play_now: true,
            reason: RoarReason::RageBuildup,
            time_override: None,
        };
    }

    // Periodic snore
    if last_roar_time > 30.0 {
        return RoarDecision {
            roar_type: RoarType::Snore,
            should_play_now: true,
            reason: RoarReason::PeriodicSnore,
            time_override: None,
        };
    }

    RoarDecision {
        roar_type: RoarType::None,
        should_play_now: false,
        reason: RoarReason::None,
        time_override: None,
    }
}

/// Debug logging for ghost state
pub(crate) fn debug_log_ghost_state(
    ghost: &GhostSprite,
    rage_result: &RageUpdateResult,
    roar_decision: &RoarDecision,
) {
    debug!(
        "Ghost calm time: {:.1}, rage: {:.1}, rage limit: {:.1}, hunting: {:.1}, warn act: {}, warning int: {:.1}, warning timer: {:.1}, roar reason: {:?}",
        ghost.calm_time_secs,
        ghost.rage,
        rage_result.rage_limit,
        ghost.hunting,
        ghost.hunt_warning_active,
        ghost.hunt_warning_intensity,
        ghost.hunt_warning_timer,
        roar_decision.reason
    );
}

/// Calculate distance with Z component multiplied by 10 if on different floors
/// This makes the ghost less effective at damaging players across floors
pub(crate) fn calculate_weighted_distance(ghost_pos: &Position, player_pos: &Position) -> f32 {
    let dx = player_pos.x - ghost_pos.x;
    let dy = player_pos.y - ghost_pos.y;

    // Check if they're on different floors by comparing rounded Z values
    let ghost_floor = ghost_pos.z.round();
    let player_floor = player_pos.z.round();

    let dz = if ghost_floor != player_floor {
        // Multiply Z component by 10 when on different floors
        (player_pos.z - ghost_pos.z) * 10.0
    } else {
        player_pos.z - ghost_pos.z
    };

    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Calculate squared distance with Z component multiplied by 10 if on different floors
pub(crate) fn calculate_weighted_distance_squared(
    ghost_pos: &Position,
    player_pos: &Position,
) -> f32 {
    let dx = player_pos.x - ghost_pos.x;
    let dy = player_pos.y - ghost_pos.y;

    // Check if they're on different floors by comparing rounded Z values
    let ghost_floor = ghost_pos.z.round();
    let player_floor = player_pos.z.round();

    let dz = if ghost_floor != player_floor {
        // Multiply Z component by 10 when on different floors
        (player_pos.z - ghost_pos.z) * 10.0
    } else {
        player_pos.z - ghost_pos.z
    };

    dx * dx + dy * dy + dz * dz
}
