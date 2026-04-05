use std::f64::consts::PI;

use bevy::prelude::*;
use unghost_core::components::logic::ghost_death::GhostDeathSignal;
use unghost_core::components::logic::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unghost_core::resources::haunt_state::HauntState;
use unspatial_core::position::Position;

use crate::systems::influence_sync::ghost_influence_visual_sync;

pub(crate) mod enrage;
pub(crate) mod movement;
pub(crate) mod roar;

use enrage::ghost_enrage;
use movement::ghost_movement;

/// Logic side of ghost entity dying.
///
/// Despawns entities once their logic-owned `GhostDeathSignal` duration elapses.
/// Presentation reacts separately by creating its own local fade timer.
pub(crate) fn ghost_dying_logic_system(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &GhostDeathSignal)>,
) {
    let current_secs = time.elapsed_secs_f64();
    for (entity, dying) in query.iter() {
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

    crate::systems::dynamic_behavior_update::app_setup(app);
    crate::systems::sound_field_pulse::app_setup(app);
}
