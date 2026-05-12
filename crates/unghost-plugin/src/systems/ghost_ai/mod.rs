use std::f64::consts::PI;

use bevy::prelude::*;
use bevy_replicon::prelude::ToClients;
use unaudiospatial_core::emitter::LocalAudioEmitter;
use unghost_core::components::logic::ghost_death::{GhostDeathSequenceState, GhostDeathSignal};
use unghost_core::components::logic::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unghost_core::events::GhostAudioMessage;
use unghost_core::resources::haunt_state::HauntState;
use unghost_core::resources::signals::{GhostHuntPressure, GhostHuntSignals, PrimaryGhostSignal};
use unspatial_core::position::Position;

use crate::systems::influence_sync::ghost_influence_visual_sync;

pub(crate) mod enrage;
pub(crate) mod movement;
pub(crate) mod roar;

use enrage::ghost_enrage;
use movement::ghost_movement;
use roar::emit_ghost_audio;

/// Logic side of ghost entity dying.
///
/// Despawns entities once their logic-owned `GhostDeathSignal` duration elapses.
/// Presentation reacts separately by creating its own local fade timer.
pub(crate) fn ghost_dying_logic_system(
    mut commands: Commands,
    time: Res<Time>,
    q_dying: Query<(Entity, &GhostDeathSignal)>,
    mut q_sequence: Query<(&GhostDeathSignal, &Position, &mut GhostDeathSequenceState)>,
    mut local_audio: LocalAudioEmitter,
    mut ev_audio: MessageWriter<ToClients<GhostAudioMessage>>,
    local_player_role: Option<Res<unreplicon_core::resources::LocalPlayerRole>>,
) {
    let current_secs = time.elapsed_secs_f64();
    let has_local_player = local_player_role.is_some();

    for (dying, position, mut sequence) in q_sequence.iter_mut() {
        if dying.is_finished(current_secs) || sequence.final_roar_sent {
            continue;
        }

        if current_secs >= dying.started_at_secs + 3.0 {
            emit_ghost_audio(
                "sounds/ghost-roar-4.ogg".to_string(),
                2.0,
                *position,
                &mut local_audio,
                &mut ev_audio,
                has_local_player,
            );
            sequence.final_roar_sent = true;
        }
    }

    for (entity, dying) in q_dying.iter() {
        if dying.is_finished(current_secs) {
            commands.entity(entity).despawn();
        }
    }
}

/// Updates the ghost warning field based on the intensity of nearby ghosts.
pub(crate) fn update_ghost_warning_field(
    mut haunt_state: ResMut<HauntState>,
    q_ghost: Query<(&GhostSprite, &Position, &GhostBehaviorDynamics)>,
    time: Res<Time>,
) {
    haunt_state.ghost_warning_intensity = 0.0;
    haunt_state.ghost_warning_position = None;
    haunt_state.evidences.clear();

    let mut max_intensity = 0.0;
    let mut main_ghost_dynamics = None;

    for (ghost, position, dynamics) in q_ghost.iter() {
        for evidence in ghost.class.evidences() {
            haunt_state.evidences.insert(evidence);
        }

        if ghost.hunt_warning_intensity > max_intensity {
            max_intensity = ghost.hunt_warning_intensity;
            haunt_state.ghost_warning_position = Some(*position);
            main_ghost_dynamics = Some(*dynamics);
            haunt_state.breach_pos = ghost.spawn_point.to_position();
        }
    }

    if main_ghost_dynamics.is_none()
        && let Some((ghost, _, dynamics)) = q_ghost.iter().next()
    {
        main_ghost_dynamics = Some(*dynamics);
        haunt_state.breach_pos = ghost.spawn_point.to_position();
    }

    if let Some(dynamics) = main_ghost_dynamics {
        haunt_state.ghost_dynamics = dynamics;
    }

    let cur_t = time.elapsed_secs_f64();
    let wave = f64::sin(PI * cur_t * 2.0).powi(2);
    haunt_state.ghost_warning_intensity = max_intensity * wave as f32;
}

pub(crate) fn refresh_ghost_hunt_signals(
    mut signals: ResMut<GhostHuntSignals>,
    q_ghost: Query<(&GhostSprite, &Position)>,
) {
    signals.any_present = false;
    signals.any_hunting = false;
    signals.any_warning_active = false;
    signals.any_hunted_this_mission = false;
    signals.any_hunt_likely = false;
    signals.any_near_hunt_without_warning = false;
    signals.primary = None;
    signals.pressures.clear();

    for (ghost, position) in q_ghost.iter() {
        signals.any_present = true;
        if signals.primary.is_none() {
            signals.primary = Some(PrimaryGhostSignal {
                class: ghost.class,
                position: *position,
                spawn_point: ghost.spawn_point.clone(),
                health: ghost.get_health(),
                hunting: ghost.hunting,
                hunt_warning_active: ghost.hunt_warning_active,
                repellent_hits: ghost.repellent_hits,
            });
        }

        if ghost.hunt_warning_active && ghost.get_health() > 0.3 {
            signals.any_warning_active = true;
        }

        if ghost.times_hunted_this_mission > 0 {
            signals.any_hunted_this_mission = true;
        }

        let rage_ratio = if ghost.rage_limit > 0.0 {
            ghost.rage / ghost.rage_limit
        } else {
            0.0
        };

        if ghost.hunt_warning_active || rage_ratio > 0.70 {
            signals.any_hunt_likely = true;
        }

        if rage_ratio > 0.80 && !ghost.hunt_warning_active && !ghost.hunt_target {
            signals.any_near_hunt_without_warning = true;
        }

        if !ghost.hunt_target {
            continue;
        }

        signals.any_hunting = true;
        signals.pressures.push(GhostHuntPressure {
            position: *position,
            calm_time_secs: ghost.calm_time_secs,
            is_warping: ghost.warp > 0.1,
        });
    }
}

pub(crate) fn app_setup(app: &mut App) {
    use unmission_core::types::SimulationState;

    app.add_systems(
        Update,
        (
            ghost_movement.run_if(resource_exists::<unreplicon_core::resources::AuthorityRole>),
            ghost_enrage.run_if(resource_exists::<unreplicon_core::resources::AuthorityRole>),
            ghost_dying_logic_system
                .run_if(resource_exists::<unreplicon_core::resources::AuthorityRole>),
            update_ghost_warning_field,
        )
            .run_if(in_state(SimulationState::Ready)),
    );

    app.add_systems(
        Update,
        ghost_influence_visual_sync.run_if(in_state(SimulationState::Ready)),
    );

    app.add_systems(
        PostUpdate,
        refresh_ghost_hunt_signals.run_if(in_state(SimulationState::Ready)),
    );

    crate::systems::dynamic_behavior_update::app_setup(app);
    crate::systems::sound_field_pulse::app_setup(app);
}
mod reproduction_test;
