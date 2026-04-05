use bevy::prelude::*;
use rand::RngExt;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::entity::GameSprite;
use uncommon_app_core::random_seed;
use unghost_core::components::logic::interaction::{InteractionMotion, Locked};
use unghost_core::components::presentation::interaction::{
    InteractionParticle, InteractionParticleType, MotionBlur, Tween, TweenEase,
};
use unmetrics_core::metrics::SendMetric;
use unrender_std::components::sprite_layer::SpriteLayer;
use unrender_std::custom_material1::CustomMaterial1;
use unspatial_core::position::Position;

use crate::components::interaction::LockIndicator;
use crate::metrics;

pub(crate) fn app_setup(app: &mut App) {
    use unreplicon_core::resources::LocalPlayerRole;
    app.add_systems(
        bevy::prelude::Update,
        (
            sync_local_tweens_system,
            ensure_motion_blur_system,
            spawn_interaction_particles_system.run_if(resource_exists::<LocalPlayerRole>),
            motion_blur_system,
            cleanup_finished_local_tweens_system,
            update_interaction_particles_system,
            door_lock_indicator_system.run_if(resource_exists::<LocalPlayerRole>),
        ),
    );
}

fn sync_local_tweens_system(
    mut commands: Commands,
    q_new_motion: Query<(Entity, &InteractionMotion), Added<InteractionMotion>>,
) {
    for (entity, motion) in q_new_motion.iter() {
        commands.entity(entity).insert(Tween::from_motion(motion));
    }
}

fn ensure_motion_blur_system(
    mut commands: Commands,
    q_new_tweens: Query<(Entity, &Position, &Tween), Added<Tween>>,
    q_motion_blur: Query<&MotionBlur>,
) {
    for (entity, position, tween) in q_new_tweens.iter() {
        if tween.ease_fn == TweenEase::ParabolicArc && q_motion_blur.get(entity).is_err() {
            commands.entity(entity).insert(MotionBlur {
                intensity: 0.0,
                previous_position: *position,
            });
        }
    }
}

fn cleanup_finished_local_tweens_system(
    mut commands: Commands,
    time: Res<Time>,
    q_tweens: Query<(Entity, &Tween)>,
) {
    let current_secs = time.elapsed_secs_f64();
    for (entity, tween) in q_tweens.iter() {
        if tween.is_finished(current_secs) {
            commands.entity(entity).remove::<Tween>();
        }
    }
}

fn spawn_interaction_particles_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    q_tweens: Query<(Entity, &Position, &Tween), Changed<Tween>>,
) {
    let measure = metrics::GIS_SPAWN_PARTICLES.time_measure();
    let current_secs = time.elapsed_secs_f64();
    for (entity, position, tween) in q_tweens.iter() {
        let progress = tween.fraction(current_secs);

        match tween.ease_fn {
            TweenEase::ParabolicArc => {
                if progress < 0.9 {
                    spawn_trail_particle(&mut commands, &asset_server, *position, entity);
                }
                if progress > 0.85 && progress < 0.95 {
                    spawn_dust_particles(&mut commands, &asset_server, tween.end_pos, 3);
                }
            }
            TweenEase::Linear => {
                if progress < 0.8 && time.elapsed_secs() % 0.3 < 0.1 {
                    spawn_haunted_glow_particle(&mut commands, &asset_server, *position);
                }
            }
            TweenEase::SineEaseOut => {
                if progress > 0.7 && progress < 0.8 {
                    spawn_dust_particles(&mut commands, &asset_server, *position, 1);
                }
            }
        }
    }
    measure.end_ms();
}

fn motion_blur_system(
    mut commands: Commands,
    mut q_blur: Query<(
        Entity,
        &Position,
        &mut MotionBlur,
        Option<&mut Sprite>,
        Option<&MeshMaterial2d<CustomMaterial1>>,
    )>,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
    q_tweens: Query<&Tween>,
) {
    let measure = metrics::GIS_MOTION_BLUR.time_measure();
    for (entity, position, mut motion_blur, mut sprite, mat) in q_blur.iter_mut() {
        let movement_delta = position.distance(&motion_blur.previous_position);

        let new_alpha = if movement_delta > 0.1 {
            motion_blur.intensity = (movement_delta * 10.0).clamp(0.0, 1.0);
            0.7 - motion_blur.intensity * 0.3
        } else {
            motion_blur.intensity *= 0.9;
            1.0 - motion_blur.intensity * 0.3
        };

        let mut final_alpha = new_alpha;

        if q_tweens.get(entity).is_err() {
            commands.entity(entity).remove::<MotionBlur>();
            final_alpha = 1.0;
        }

        if let Some(sprite) = sprite.as_mut() {
            sprite.color.set_alpha(final_alpha);
        }
        if let Some(mat) = mat
            && let Some(material) = materials1.get_mut(mat)
        {
            material.data.color.set_alpha(final_alpha);
        }

        motion_blur.previous_position = *position;
    }
    measure.end_ms();
}

fn update_interaction_particles_system(
    mut commands: Commands,
    time: Res<Time>,
    mut q_particles: Query<(
        Entity,
        &mut Position,
        &mut InteractionParticle,
        &mut MapColor,
        &mut Transform,
    )>,
) {
    let measure = metrics::GIS_UPDATE_PARTICLES.time_measure();
    for (entity, mut position, mut particle, mut map_color, mut transform) in q_particles.iter_mut()
    {
        particle.life -= time.delta_secs();

        if particle.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        position.x += particle.velocity.x * time.delta_secs();
        position.y += particle.velocity.y * time.delta_secs();
        position.z += particle.velocity.z * time.delta_secs();

        let life_ratio = particle.life / particle.max_life;

        match particle.particle_type {
            InteractionParticleType::Dust => {
                particle.velocity.z -= 0.5 * time.delta_secs();
                particle.velocity.x *= 0.98;
                particle.velocity.y *= 0.98;
                map_color.color.set_alpha(life_ratio * 0.6);
                transform.scale = Vec3::splat(0.3 + life_ratio * 0.2);
            }
            InteractionParticleType::Trail => {
                map_color.color.set_alpha(life_ratio * 0.4);
                transform.scale = Vec3::splat(0.2 + life_ratio * 0.1);
            }
            InteractionParticleType::HauntedGlow => {
                particle.velocity.z += 0.2 * time.delta_secs();
                let pulse = (time.elapsed_secs() * 3.0).sin() * 0.2 + 0.8;
                map_color.color.set_alpha(life_ratio * 0.7 * pulse);
                transform.scale = Vec3::splat(0.4 + pulse * 0.2);
            }
            InteractionParticleType::Spark => {
                particle.velocity.z -= 2.0 * time.delta_secs();
                let flicker = if (time.elapsed_secs() * 10.0) % 1.0 > 0.5 {
                    1.0
                } else {
                    0.3
                };
                map_color.color.set_alpha(life_ratio * flicker);
                transform.scale = Vec3::splat(0.1 + life_ratio * 0.1);
            }
        }
    }
    measure.end_ms();
}

fn door_lock_indicator_system(
    mut commands: Commands,
    time: Res<Time>,
    _asset_server: Res<AssetServer>,
    mut q_indicators: Query<(Entity, &mut LockIndicator, &mut Sprite)>,
    q_locked_visual_targets: Query<Entity, (Added<Locked>, With<Sprite>)>,
    q_locked_without_sprite: Query<Entity, (Added<Locked>, Without<Sprite>)>,
    q_unlocked_doors: Query<Entity, (With<LockIndicator>, Without<Locked>)>,
) {
    let measure = metrics::GIS_DOOR_LOCK_INDICATOR.time_measure();
    for entity in q_locked_without_sprite.iter() {
        debug!(
            "door_lock_indicator_system: skipping LockIndicator for locked entity {:?} without Sprite",
            entity
        );
    }
    for door_entity in q_locked_visual_targets.iter() {
        commands.entity(door_entity).insert(LockIndicator::new());
    }
    for door_entity in q_unlocked_doors.iter() {
        commands.entity(door_entity).remove::<LockIndicator>();
    }
    for (_entity, mut indicator, mut sprite) in q_indicators.iter_mut() {
        indicator.pulse_timer.tick(time.delta());
        let pulse_progress = indicator.pulse_timer.fraction();
        let pulse_alpha = (pulse_progress * std::f32::consts::PI * 2.0).sin() * 0.3 + 0.7;
        sprite.color = Color::srgba(1.0, 0.7, 0.7, indicator.base_alpha * pulse_alpha);
    }
    measure.end_ms();
}

fn spawn_trail_particle(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Position,
    _parent_entity: Entity,
) {
    let mut rng = random_seed::rng();

    commands
        .spawn(Sprite {
            image: asset_server.load("img/particle_small.png"),
            color: Color::srgba(0.8, 0.8, 0.9, 0.4),
            custom_size: Some(Vec2::new(0.1, 0.1)),
            ..default()
        })
        .insert(position)
        .insert(GameSprite)
        .insert(MapColor {
            color: Color::srgba(0.8, 0.8, 0.9, 0.4),
        })
        .insert(SpriteLayer::default())
        .insert(InteractionParticle {
            life: 0.5,
            max_life: 0.5,
            velocity: Vec3::new(
                rng.random_range(-0.1..0.1),
                rng.random_range(-0.1..0.1),
                rng.random_range(0.0..0.1),
            ),
            particle_type: InteractionParticleType::Trail,
        })
        .insert(Transform::from_scale(Vec3::splat(0.2)));
}

fn spawn_dust_particles(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Position,
    count: usize,
) {
    let mut rng = random_seed::rng();

    for _ in 0..count {
        commands
            .spawn(Sprite {
                image: asset_server.load("img/particle_dust.png"),
                color: Color::srgba(0.6, 0.5, 0.4, 0.6),
                custom_size: Some(Vec2::new(0.2, 0.2)),
                ..default()
            })
            .insert(Position {
                x: position.x + rng.random_range(-0.15..0.15),
                y: position.y + rng.random_range(-0.15..0.15),
                z: position.z + rng.random_range(0.0..0.1),
                visual_priority: position.visual_priority,
            })
            .insert(GameSprite)
            .insert(MapColor {
                color: Color::srgba(0.6, 0.5, 0.4, 0.6),
            })
            .insert(SpriteLayer::default())
            .insert(InteractionParticle {
                life: 1.5,
                max_life: 1.5,
                velocity: Vec3::new(
                    rng.random_range(-0.2..0.2),
                    rng.random_range(-0.2..0.2),
                    rng.random_range(0.1..0.4),
                ),
                particle_type: InteractionParticleType::Dust,
            })
            .insert(Transform::from_scale(Vec3::splat(0.4)));
    }
}

fn spawn_haunted_glow_particle(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Position,
) {
    let mut rng = random_seed::rng();

    commands
        .spawn(Sprite {
            image: asset_server.load("img/particle_glow.png"),
            color: Color::srgba(0.3, 0.8, 0.3, 0.7),
            custom_size: Some(Vec2::new(0.3, 0.3)),
            ..default()
        })
        .insert(Position {
            x: position.x + rng.random_range(-0.2..0.2),
            y: position.y + rng.random_range(-0.2..0.2),
            z: position.z + rng.random_range(0.0..0.2),
            visual_priority: position.visual_priority,
        })
        .insert(GameSprite)
        .insert(MapColor {
            color: Color::srgba(0.3, 0.8, 0.3, 0.7),
        })
        .insert(SpriteLayer::default())
        .insert(InteractionParticle {
            life: 2.0,
            max_life: 2.0,
            velocity: Vec3::new(
                rng.random_range(-0.05..0.05),
                rng.random_range(-0.05..0.05),
                0.1,
            ),
            particle_type: InteractionParticleType::HauntedGlow,
        })
        .insert(Transform::from_scale(Vec3::splat(0.5)));
}
