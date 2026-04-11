use std::time::Duration;

use bevy::prelude::*;
use rand::prelude::*;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::entity::GameSprite;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use uncommon_app_core::random_seed;
use uncommon_states_core::UIContextState;
use unghost_core::components::logic::ghost_breach::GhostBreach;
use unghost_core::components::logic::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unghost_core::components::presentation::ghost_orb_particle::GhostOrbParticle;
use unlight_core::spectral::SpectralInfluence;
use unrender_std::components::sprite_layer::SpriteLayer;
use unreplicon_core::resources::LocalPlayerRole;
use unspatial_core::position::Position;

/// Timer resource for controlling orb spawn rate (~1 per second)
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
fn spawn_ghost_orb_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<OrbSpawnTimer>,
    breach_query: Query<(Entity, &Position), With<GhostBreach>>,
    ghost_query: Query<(&GhostSprite, &GhostBehaviorDynamics)>,
) {
    let mut rng = random_seed::rng();
    spawn_timer.0.tick(time.delta());

    if !spawn_timer.0.just_finished() {
        return;
    }

    for (breach_entity, breach_pos) in breach_query.iter() {
        let Some((_, dynamics)) = ghost_query
            .iter()
            .find(|(g, _)| g.breach_id == Some(breach_entity))
        else {
            continue;
        };

        if !rng.random_bool(dynamics.floating_orbs_clarity.clamp(0.0, 1.0).cbrt() as f64) {
            continue;
        }

        let mut base_position = breach_pos.to_vec3();
        // Lower the base Z to prevent it from clipping into the floor above
        base_position.z += 0.15;

        // 10% chance to be wandering, taking it off the breach and looking for paths
        let base_speed = rng.random_range(0.0..1.0_f32).powf(3.0);
        let is_moving = base_speed > 0.05; // If it's very slow, just let it be a traditional static orb

        let life = if is_moving {
            rng.random_range(30.0..60.0)
        } else {
            5.0
        };

        commands
            .spawn(Sprite {
                color: Color::WHITE.with_alpha(0.0),
                custom_size: Some(Vec2::new(1.0, 1.0)),
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
                life,
                time.elapsed_secs(),
                base_position,
                base_speed,
            ));
    }
}

/// Updates ghost orb particles (movement, collision, lifecycle)
fn update_ghost_orb_particles(
    mut commands: Commands,
    time: Res<Time>,
    board_topology: Res<BoardTopology>,
    board_collision: Res<BoardCollisionField>,
    thermal_grid: Option<Res<unthermal_core::resources::ThermalGrid>>,
    noise_table: Res<unnoise_core::perlin::PerlinNoise>,
    mut query: Query<
        (
            Entity,
            &mut Position,
            &mut GhostOrbParticle,
            Option<&mut MapColor>,
        ),
        With<GhostOrbParticle>,
    >,
) {
    for (entity, mut position, mut particle, mut map_color) in query.iter_mut() {
        particle.life -= time.delta_secs();

        if particle.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let elapsed_time = time.elapsed_secs() - particle.initial_spawn_time;

        // Visibility driven by perlin noise (slowly fading in and out)
        // We use a different slice of phase to avoid linking visibility to the wobble path
        let vis_noise = noise_table.get(
            particle.phase.y + elapsed_time * 0.15,
            particle.phase.z - elapsed_time * 0.1,
        );
        // Map noise (-1 to 1) so that only peaks > 0.4 are visible. This drastically
        // reduces the amount of visible orbs to about ~1/10th or 1/20th at any given time.
        let noise_alpha = ((vis_noise - 0.4) * 3.0).clamp(0.0, 1.0);

        let life_fade = (particle.life / 2.0)
            .clamp(0.0, 1.0)
            .min((elapsed_time / 2.0).clamp(0.0, 1.0));

        let final_alpha = noise_alpha * life_fade;

        // Modulate MapColor's alpha so the lighting pipeline properly scales the final opacity.
        // Doing this preserves the green nightvision tints and wall occlusion logic.
        if let Some(mc) = map_color.as_mut() {
            mc.color = Color::WHITE.with_alpha(final_alpha);
        }

        if particle.speed_multiplier > 0.05 {
            let mut gradient = Vec2::ZERO;

            if let Some(tg) = thermal_grid.as_deref() {
                let bp = Position {
                    x: particle.base_position.x,
                    y: particle.base_position.y,
                    z: particle.base_position.z,
                    visual_priority: position.visual_priority,
                }
                .to_board_position();

                if let Some(cidx) = bp.ndidx_checked(board_topology.map_size) {
                    let p_center = tg.temperature_field[cidx];

                    let get_temp = |p: unspatial_core::boardposition::BoardPosition| {
                        if let Some(idx) = p.ndidx_checked(board_topology.map_size)
                            && board_collision.0[idx].ghost_free
                        {
                            return tg.temperature_field[idx];
                        }
                        p_center
                    };

                    let t_left = get_temp(bp.left());
                    let t_right = get_temp(bp.right());
                    let t_top = get_temp(bp.top());
                    let t_bottom = get_temp(bp.bottom());

                    // Gradient points towards warmer temperatures
                    let grad_x = t_right - t_left;
                    let grad_y = t_top - t_bottom;

                    gradient = Vec2::new(grad_x, grad_y);
                }
            }

            // Drastically reduced movement noise frequency as requested
            let noise_x = noise_table.get(particle.phase.x + elapsed_time * 0.02, particle.phase.y);
            let noise_y = noise_table.get(particle.phase.y, particle.phase.z + elapsed_time * 0.02);
            let noise_vec = Vec2::new(noise_x, noise_y);

            // Combine gradient and noise for steering force (Boids approach)
            let steering_force = gradient * 20.0 + noise_vec * 0.5;

            // Apply steering force to velocity (inertia)
            particle.drift_velocity += steering_force * time.delta_secs();

            // Normalize momentum to guarantee they never slow down or stop (Constant flight)
            let current_speed = particle.drift_velocity.length();
            if current_speed > 0.001 {
                particle.drift_velocity /= current_speed;
            } else {
                particle.drift_velocity = Vec2::X; // Fallback
            }

            // Move the particle, preserving full momentum
            let ds = particle.drift_velocity * time.delta_secs() * particle.speed_multiplier * 1.5;

            particle.base_position.x += ds.x;
            particle.base_position.y += ds.y;

            let current_bpos = Position {
                x: particle.base_position.x,
                y: particle.base_position.y,
                z: particle.base_position.z,
                visual_priority: position.visual_priority,
            }
            .to_board_position();

            // Soft circular collision against walls, identical to player/gas movement
            for npos in current_bpos.iter_xy_neighbors_nosize(1) {
                if npos
                    .ndidx_checked(board_topology.map_size)
                    .is_some_and(|idx| !board_collision.0[idx].ghost_free)
                {
                    let tile_center = npos.to_position();
                    let dpos = Vec2::new(
                        tile_center.x - particle.base_position.x,
                        tile_center.y - particle.base_position.y,
                    );

                    let mut dapos = dpos.abs();
                    dapos.x -= 0.3; // Half-size of the "pillar"
                    dapos.y -= 0.3;
                    dapos.x = dapos.x.max(0.0);
                    dapos.y = dapos.y.max(0.0);

                    let ddist = dapos.distance(Vec2::ZERO);
                    let radius = 0.5; // Particle soft radius

                    if ddist < radius {
                        if dpos.x < 0.0 {
                            dapos.x *= -1.0;
                        }
                        if dpos.y < 0.0 {
                            dapos.y *= -1.0;
                        }

                        // Push base position smoothly out of the wall
                        let fix_dist = (radius - ddist).powi(2);
                        let push = dapos / (ddist + 0.000001) * fix_dist;

                        particle.base_position.x -= push.x;
                        particle.base_position.y -= push.y;

                        // Deflect velocity to slide along the shape of the wall
                        particle.drift_velocity.x -= push.x * 2.0;
                        particle.drift_velocity.y -= push.y * 2.0;
                    }
                }
            }
        }

        let dx =
            particle.amplitude.x * (particle.frequency.x * elapsed_time + particle.phase.x).sin();
        let dy =
            particle.amplitude.y * (particle.frequency.y * elapsed_time + particle.phase.y).sin();
        let dz =
            particle.amplitude.z * (particle.frequency.z * elapsed_time + particle.phase.z).sin();

        let target_x = particle.base_position.x + dx;
        let target_y = particle.base_position.y + dy;
        let target_z = particle.base_position.z + dz;

        let next_pos = Position {
            x: target_x,
            y: target_y,
            z: target_z,
            visual_priority: position.visual_priority,
        };

        let next_bpos = next_pos.to_board_position();

        // Assign the final position directly. Soft collision on the base_position
        // ensures the center mass stays out of walls, while this lets the sinusoidal
        // offset bleed slightly into walls visually without jittering.
        if next_bpos.ndidx_checked(board_topology.map_size).is_some() {
            *position = next_pos;
            position.z = position
                .z
                .clamp(next_bpos.z as f32 + 0.05, next_bpos.z as f32 + 0.95);
        } else {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (spawn_ghost_orb_particles, update_ghost_orb_particles)
            .run_if(in_state(UIContextState::InGame))
            .run_if(resource_exists::<LocalPlayerRole>),
    );
}
