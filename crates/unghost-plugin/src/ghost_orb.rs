use std::{f32::consts::TAU, time::Duration};

use bevy::prelude::*;
use rand::prelude::*;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use unfoundation_core::random_seed;
use unghost_core::components::ghost_breach::GhostBreach;
use unghost_core::components::ghost_orb_particle::GhostOrbParticle;
use unghost_core::components::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unrender_std::components::game::GameSprite;
use unrender_std::components::sprite_layer::SpriteLayer;
use unsensing_core::components::SpectralInfluence;
use unspatial_core::position::Position;

// Timer resource for controlling orb spawn rate (~1 per second)
#[derive(Resource)]
pub(crate) struct OrbSpawnTimer(pub Timer);

impl Default for OrbSpawnTimer {
    fn default() -> Self {
        OrbSpawnTimer(Timer::new(
            Duration::from_secs_f32(0.2),
            TimerMode::Repeating,
        ))
    }
}

/// Spawns ghost orb particles from ghost breaches if the FloatingOrbs evidence is active
pub(crate) fn spawn_ghost_orb_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<OrbSpawnTimer>,
    breach_query: Query<(Entity, &Position), With<GhostBreach>>,
    ghost_query: Query<(&GhostSprite, &GhostBehaviorDynamics)>,
) {
    let mut rng = random_seed::rng();
    spawn_timer.0.tick(time.delta());

    // Only proceed if the timer finished
    if !spawn_timer.0.just_finished() {
        return;
    }

    // For each ghost breach
    for (breach_entity, breach_pos) in breach_query.iter() {
        // Find the ghost associated with this breach
        let Some((ghost, dynamics)) = ghost_query
            .iter()
            .find(|(g, _)| g.breach_id == Some(breach_entity))
        else {
            continue;
        };

        // If the ghost doesn't have FloatingOrbs evidence, skip
        if !ghost
            .class
            .evidences()
            .contains(&unghost_core::types::evidence::Evidence::FloatingOrbs)
        {
            continue;
        }

        // Random check based on clarity
        if !rng.random_bool(dynamics.floating_orbs_clarity.clamp(0.0, 1.0).cbrt() as f64) {
            continue;
        }

        // Convert to Vec3 for base position
        let mut base_position = breach_pos.to_vec3();
        base_position.z += 0.45;

        // Spawn a new ghost orb particle
        commands
            .spawn(Sprite {
                color: Color::WHITE.with_alpha(0.0),
                custom_size: Some(Vec2::new(1.0, 1.0)), // Small size for "pixel" appearance
                ..default()
            })
            .insert(Position {
                x: base_position.x,
                y: base_position.y,
                z: base_position.z,
                visual_priority: breach_pos.visual_priority,
            })
            .insert(GameSprite)
            .insert(SpriteLayer(5.0))
            .insert(MapColor {
                color: Color::WHITE,
            })
            .insert(SpectralInfluence::default().with_infrared(1.0, Some(0.5)))
            .insert(GhostOrbParticle::new(
                5.0, // 5 second lifetime
                time.elapsed_secs(),
                base_position,
            ));
    }
}

/// Updates ghost orb particles (movement, collision, lifecycle)
pub(crate) fn update_ghost_orb_particles(
    mut commands: Commands,
    time: Res<Time>,
    board_topology: Res<BoardTopology>,
    board_collision: Res<BoardCollisionField>,
    mut query: Query<(Entity, &mut Position, &mut GhostOrbParticle), With<GhostOrbParticle>>,
) {
    for (entity, mut position, mut particle) in query.iter_mut() {
        // Update lifetime
        particle.life -= time.delta_secs();

        // Despawn if lifetime is over
        if particle.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Calculate sinusoidal movement
        let elapsed_time = time.elapsed_secs() - particle.initial_spawn_time;

        // Calculate offsets using sine waves with different frequencies and phases
        let dx =
            particle.amplitude.x * (particle.frequency.x * elapsed_time + particle.phase.x).sin();
        let dy =
            particle.amplitude.y * (particle.frequency.y * elapsed_time + particle.phase.y).sin();
        let dz =
            particle.amplitude.z * (particle.frequency.z * elapsed_time + particle.phase.z).sin();

        // Calculate target position
        let target_x = particle.base_position.x + dx;
        let target_y = particle.base_position.y + dy;
        let target_z = particle.base_position.z + dz;

        // Calculate next position
        let next_pos = Position {
            // Remove mutability from next_pos
            x: target_x,
            y: target_y,
            z: target_z,
            visual_priority: position.visual_priority, // Preserve visual_priority
        };

        // Get board position for collision detection
        let next_bpos = next_pos.to_board_position();

        // Check if next position is within board bounds and is free to move into
        if let Some(idx) = next_bpos.ndidx_checked(board_topology.map_size) {
            if board_collision.0[idx].player_free {
                // Move to the new position if it's valid
                *position = next_pos;
            } else {
                // Handle collision with walls - simple reflection by reversing the phase
                // Check each axis separately for more natural movement

                // X-axis collision
                let x_check_bpos = Position {
                    x: target_x,
                    y: position.y,
                    z: position.z,
                    visual_priority: position.visual_priority,
                }
                .to_board_position();

                if let Some(x_idx) = x_check_bpos.ndidx_checked(board_topology.map_size) {
                    if board_collision.0[x_idx].player_free {
                        position.x = target_x;
                    } else {
                        // Reverse x direction by adjusting phase
                        particle.phase.x += TAU / 2.0; // Add half a cycle (π)
                    }
                }

                // Y-axis collision
                let y_check_bpos = Position {
                    x: position.x,
                    y: target_y,
                    z: position.z,
                    visual_priority: position.visual_priority,
                }
                .to_board_position();

                if let Some(y_idx) = y_check_bpos.ndidx_checked(board_topology.map_size) {
                    if board_collision.0[y_idx].player_free {
                        position.y = target_y;
                    } else {
                        // Reverse y direction by adjusting phase
                        particle.phase.y += TAU / 2.0; // Add half a cycle (π)
                    }
                }

                // Z-axis - make sure it's within reasonable bounds
                position.z = target_z.clamp(
                    next_bpos.z as f32 + 0.05, // Slightly above floor
                    next_bpos.z as f32 + 0.95, // Below ceiling
                );
            }
        } else {
            // Out of board bounds, despawn the particle
            commands.entity(entity).despawn();
        }
    }
}

/// Sets up the ghost orb systems in the app
pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<OrbSpawnTimer>().add_systems(
        Update,
        (spawn_ghost_orb_particles, update_ghost_orb_particles),
    );
}
