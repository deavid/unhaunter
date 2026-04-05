use std::{f32::consts::TAU, time::Duration};

use bevy::prelude::*;
use rand::prelude::*;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::entity::GameSprite;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use uncommon_app_core::random_seed;
use unghost_core::components::logic::ghost_breach::GhostBreach;
use unghost_core::components::logic::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unghost_core::components::presentation::ghost_orb_particle::GhostOrbParticle;
use unlight_core::spectral::SpectralInfluence;
use unorchestrator_core::UIContextState;
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
        base_position.z += 0.45;

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
                5.0,
                time.elapsed_secs(),
                base_position,
            ));
    }
}

/// Updates ghost orb particles (movement, collision, lifecycle)
fn update_ghost_orb_particles(
    mut commands: Commands,
    time: Res<Time>,
    board_topology: Res<BoardTopology>,
    board_collision: Res<BoardCollisionField>,
    mut query: Query<(Entity, &mut Position, &mut GhostOrbParticle), With<GhostOrbParticle>>,
) {
    for (entity, mut position, mut particle) in query.iter_mut() {
        particle.life -= time.delta_secs();

        if particle.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let elapsed_time = time.elapsed_secs() - particle.initial_spawn_time;

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

        if let Some(idx) = next_bpos.ndidx_checked(board_topology.map_size) {
            if board_collision.0[idx].player_free {
                *position = next_pos;
            } else {
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
                        particle.phase.x += TAU / 2.0;
                    }
                }

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
                        particle.phase.y += TAU / 2.0;
                    }
                }

                position.z = target_z.clamp(next_bpos.z as f32 + 0.05, next_bpos.z as f32 + 0.95);
            }
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
