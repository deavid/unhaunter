use std::{f32::consts::TAU, time::Duration};

use bevy::prelude::*;
use rand::Rng; // Import the Rng trait
use uncore::{
    components::{
        board::{mapcolor::MapColor, position::Position},
        game::GameSprite,
        ghost_breach::GhostBreach,
        ghost_orb_particle::GhostOrbParticle,
        sprite_type::SpriteType,
    },
    random_seed,
    resources::board_data::BoardData,
};

// Timer resource for controlling orb spawn rate (~1 per second)
#[derive(Resource)]
pub struct OrbSpawnTimer(pub Timer);

impl Default for OrbSpawnTimer {
    fn default() -> Self {
        OrbSpawnTimer(Timer::new(
            Duration::from_secs_f32(0.2),
            TimerMode::Repeating,
        ))
    }
}

/// Spawns ghost orb particles from ghost breaches if the FloatingOrbs evidence is active
pub fn spawn_ghost_orb_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<OrbSpawnTimer>,
    breach_query: Query<(Entity, &Position), With<GhostBreach>>,
    board_data: Res<BoardData>,
) {
    let mut rng = random_seed::rng();
    spawn_timer.0.tick(time.delta());

    // Only spawn orbs if the timer finished and FloatingOrbs is an active evidence type
    if !spawn_timer.0.just_finished()
        || !rng.random_bool(
            board_data
                .ghost_dynamics
                .floating_orbs_clarity
                .clamp(0.0, 1.0)
                .cbrt() as f64,
        )
    {
        return;
    }

    // For each ghost breach
    for (_breach_entity, breach_pos) in breach_query.iter() {
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
                global_z: breach_pos.global_z,
            })
            .insert(GameSprite)
            .insert(MapColor {
                color: Color::WHITE,
            })
            .insert(SpriteType::GhostOrb)
            .insert(GhostOrbParticle::new(
                5.0, // 5 second lifetime
                time.elapsed_secs(),
                base_position,
            ));
    }
}

/// Updates ghost orb particles (movement, collision, lifecycle)
pub fn update_ghost_orb_particles(
    mut commands: Commands,
    time: Res<Time>,
    board_data: Res<BoardData>,
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
            global_z: position.global_z, // Preserve global_z
        };

        // Get board position for collision detection
        let next_bpos = next_pos.to_board_position();

        // Check if next position is within board bounds and is free to move into
        if let Some(idx) = next_bpos.ndidx_checked(board_data.map_size) {
            if board_data.collision_field[idx].player_free {
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
                    global_z: position.global_z,
                }
                .to_board_position();

                if let Some(x_idx) = x_check_bpos.ndidx_checked(board_data.map_size) {
                    if board_data.collision_field[x_idx].player_free {
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
                    global_z: position.global_z,
                }
                .to_board_position();

                if let Some(y_idx) = y_check_bpos.ndidx_checked(board_data.map_size) {
                    if board_data.collision_field[y_idx].player_free {
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
pub fn app_setup(app: &mut App) {
    app.init_resource::<OrbSpawnTimer>().add_systems(
        Update,
        (spawn_ghost_orb_particles, update_ghost_orb_particles),
    );
}
