use std::f64::consts::PI;

use bevy::prelude::*;
use bevy_replicon::prelude::{SendMode, ToClients};
use rand::prelude::*;
use unaudiospatial_core::emitter::AudioEmitter;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::entity::ResolutionFactor;
use uncommon_app_core::random_seed;
use unghost_core::components::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unghost_core::resources::haunt_state::HauntState;
use unreplicon_core::messages::SpawnParticleNetEvent;
use unspatial_core::position::Position;

use crate::components::fade_out::FadeOut;
use crate::systems::visual_sync::{ghost_influence_visual_sync, ghost_visual_sync};

pub(crate) mod enrage;
pub(crate) mod movement;
pub(crate) mod roar;

use enrage::ghost_enrage;
use movement::ghost_movement;
use roar::RoarType;

pub(crate) fn ghost_fade_out_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut FadeOut,
        &mut MapColor,
        &Position,
        Option<&GhostSprite>,
        Option<&mut GhostBehaviorDynamics>,
    )>,
    mut ga: AudioEmitter,
    mut ev_particles: MessageWriter<ToClients<SpawnParticleNetEvent>>,
) {
    let mut rng = random_seed::rng();
    for (entity, mut fade_out, mut map_color, position, ghost_sprite, o_dynamics) in
        query.iter_mut()
    {
        fade_out.timer.tick(ga.time.delta());
        let rem_f = fade_out.timer.remaining_secs() / fade_out.timer.duration().as_secs_f32();

        // Fade out the sprite
        map_color.color.set_alpha(rem_f);

        if let Some(mut dynamics) = o_dynamics {
            dynamics.visual_alpha_multiplier = rem_f;
        }

        // Emit smoke particles while fading
        if fade_out.timer.remaining_secs() > 0.0 && rng.random_bool(((1.0 - rem_f) / 3.0) as f64) {
            let pos = *position;
            ev_particles.write(ToClients {
                mode: SendMode::Broadcast,
                message: SpawnParticleNetEvent {
                    particle_type: "smoke".to_string(),
                    position: [pos.x, pos.y, pos.z],
                },
            });
        }

        // Play roar sounds
        if let Some(_ghost_sprite) = ghost_sprite {
            if !fade_out.roared {
                // Play the first roar at 100% volume
                if let Some(roar_sound) = RoarType::Full.get_sound() {
                    ga.play_audio(roar_sound, 1.0, position);
                }
                fade_out.roared = true;
            } else if fade_out.timer.is_finished() {
                // Play the second roar at a lower volume
                if let Some(roar_sound) = RoarType::Full.get_sound() {
                    ga.play_audio(roar_sound, 0.2, position);
                }

                // Despawn the entity
                commands.entity(entity).despawn();
            }
        } else if fade_out.timer.is_finished() {
            // Despawn the breach when its timer is done
            commands.entity(entity).despawn();
        }
    }
}

/// Updates the ghost warning field based on the intensity of nearby ghosts.
///
/// This system calculates the ghost warning field based on the highest intensity
/// warning from any ghost. The warning field is used to display a visual warning
/// to the player when a ghost is nearby.
pub(crate) fn update_ghost_warning_field(
    mut haunt_state: ResMut<HauntState>,
    q_ghost: Query<(&GhostSprite, &Position, &GhostBehaviorDynamics)>,
    time: Res<Time>,
) {
    // Reset warning field
    haunt_state.ghost_warning_intensity = 0.0;
    haunt_state.ghost_warning_position = None;
    haunt_state.evidences.clear();

    let mut max_intensity = 0.0;
    let mut main_ghost_dynamics = None;

    // Find the highest intensity warning from any ghost
    for (ghost, position, dynamics) in q_ghost.iter() {
        // Aggregate all evidences from all ghosts
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

    // If no ghost has triggered a warning yet, pick the first one's data for the UI
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

/// Apply a visual "glitch" effect to ghosts by modifying their Transform scale
/// This creates a visual indication of ghost instability without affecting position or movement
pub(crate) fn ghost_scale_glitch_system(
    time: Res<Time>,
    mut q_ghost: Query<
        (&GhostSprite, &mut Transform, Option<&ResolutionFactor>),
        (With<GhostSprite>, Without<FadeOut>),
    >,
) {
    let mut rng = random_seed::rng();
    let dt = time.delta_secs();

    for (ghost, mut transform, rf) in q_ghost.iter_mut() {
        let base_scale = rf.map(|r| r.ratio()).unwrap_or(1.0);
        let base_vec = Vec3::new(base_scale, base_scale, base_scale);
        if ghost.repellent_hits_delta > 0.0 {
            // Apply scale glitch based on repellent hits
            let glitch_intensity = ghost.repellent_hits_delta.clamp(0.0, 1.0);

            // Generate random scale variations
            let scale_x = base_scale + rng.random_range(-glitch_intensity..glitch_intensity) * 0.4
                - glitch_intensity * 0.1;
            let scale_y = base_scale + rng.random_range(-glitch_intensity..glitch_intensity) * 0.4;
            let scale_z = base_scale; // Keep Z scale consistent

            // Apply the glitch scale
            transform.scale = transform
                .scale
                .lerp(Vec3::new(scale_x, scale_y, scale_z), dt * 0.5);
        } else if ghost.repellent_misses_delta > 0.0 {
            let glitch_intensity = ghost.repellent_misses_delta.clamp(0.0, 1.0);
            // Generate random scale variations
            let scale_x = base_scale + glitch_intensity * 0.075;
            let scale_y = base_scale + glitch_intensity * 0.05;
            let scale_z = base_scale; // Keep Z scale consistent

            // Apply the glitch scale
            transform.scale = transform
                .scale
                .lerp(Vec3::new(scale_x, scale_y, scale_z), dt * 0.2);
        } else {
            // Restore normal scale when no glitch
            if transform.scale != base_vec {
                // Smoothly interpolate back to normal scale
                transform.scale = transform.scale.lerp(base_vec, dt * 0.5);

                // Snap to exactly base scale when very close to avoid floating point drift
                if (transform.scale - base_vec).length() < 0.01 {
                    transform.scale = base_vec;
                }
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    use unmission_core::types::SimulationState;

    app.add_systems(
        Update,
        (
            ghost_movement.run_if(resource_exists::<unreplicon_core::resources::AuthorityRole>),
            ghost_enrage.run_if(resource_exists::<unreplicon_core::resources::AuthorityRole>),
            ghost_fade_out_system
                .run_if(resource_exists::<unreplicon_core::resources::AuthorityRole>),
            update_ghost_warning_field,
            ghost_scale_glitch_system
                .run_if(resource_exists::<unreplicon_core::resources::AuthorityRole>),
        )
            .run_if(in_state(SimulationState::Ready)),
    );

    app.add_systems(
        Update,
        (ghost_visual_sync, ghost_influence_visual_sync).run_if(in_state(SimulationState::Ready)),
    );

    // Initialize dynamic behavior update system
    crate::systems::dynamic_behavior_update::app_setup(app);
    // Ghost sound field pulse (authority generates, clients receive)
    crate::systems::sound_field_pulse::app_setup(app);
}
