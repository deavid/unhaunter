use bevy::prelude::*;
use rand::Rng;
use uncore_board::components::mapcolor::MapColor;
use uncore_foundation::random_seed;
use unrender::components::game::GameSprite;
use unrender::components::sprite_type::SpriteType;
use unspatial::Position;

use crate::components::interaction::{
    InteractionParticle, InteractionParticleType, LockIndicator, Locked, MotionBlur, Tween,
    TweenEase,
};

/// Registers visual effects systems with the Bevy app
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        bevy::prelude::Update,
        (
            spawn_interaction_particles_system,
            motion_blur_system,
            update_interaction_particles_system,
            door_lock_indicator_system,
        ),
    );
}

/// System that spawns visual effect particles for object interactions
fn spawn_interaction_particles_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    q_tweens: Query<(Entity, &Position, &Tween), Changed<Tween>>,
) {
    for (entity, position, tween) in q_tweens.iter() {
        let progress = tween.timer.fraction();

        match tween.ease_fn {
            TweenEase::ParabolicArc => {
                // Spawn trail particles for thrown objects
                if progress < 0.9 {
                    spawn_trail_particle(&mut commands, &asset_server, *position, entity);
                }

                // Spawn dust particles on impact (near end of throw)
                if progress > 0.85 && progress < 0.95 {
                    spawn_dust_particles(&mut commands, &asset_server, tween.end_pos, 3);
                }
            }
            TweenEase::Linear => {
                // For haunted movement, spawn creepy glow particles
                if progress < 0.8 && time.elapsed_secs() % 0.3 < 0.1 {
                    spawn_haunted_glow_particle(&mut commands, &asset_server, *position);
                }
            }
            TweenEase::SineEaseOut => {
                // For nudges, spawn a small dust cloud at the end
                if progress > 0.7 && progress < 0.8 {
                    spawn_dust_particles(&mut commands, &asset_server, *position, 1);
                }
            }
        }
    }
}

/// System that handles motion blur effects for fast-moving objects
fn motion_blur_system(
    mut commands: Commands,
    mut q_blur: Query<(Entity, &Position, &mut MotionBlur, &mut Sprite)>,
    q_tweens: Query<&Tween>,
) {
    for (entity, position, mut motion_blur, mut sprite) in q_blur.iter_mut() {
        // Calculate movement speed
        let movement_delta = position.distance(&motion_blur.previous_position);

        // Apply blur effect based on movement speed
        if movement_delta > 0.1 {
            motion_blur.intensity = (movement_delta * 10.0).clamp(0.0, 1.0);

            // Create trail effect by adjusting alpha and scale
            sprite.color.set_alpha(0.7 - motion_blur.intensity * 0.3);
        } else {
            // Gradually reduce blur when not moving
            motion_blur.intensity *= 0.9;
            sprite.color.set_alpha(1.0 - motion_blur.intensity * 0.3);
        }

        motion_blur.previous_position = *position;

        // Remove motion blur component when tween animation is finished
        if q_tweens.get(entity).is_err() {
            commands.entity(entity).remove::<MotionBlur>();
            sprite.color.set_alpha(1.0); // Restore normal alpha
        }
    }
}

/// System that updates visual effect particles
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
    for (entity, mut position, mut particle, mut map_color, mut transform) in q_particles.iter_mut()
    {
        // Update particle lifetime
        particle.life -= time.delta_secs();

        // Despawn if lifetime is over
        if particle.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Update particle position based on velocity
        position.x += particle.velocity.x * time.delta_secs();
        position.y += particle.velocity.y * time.delta_secs();
        position.z += particle.velocity.z * time.delta_secs();

        // Update visual properties based on particle type and age
        let life_ratio = particle.life / particle.max_life;

        match particle.particle_type {
            InteractionParticleType::Dust => {
                // Dust particles fade out and fall down
                particle.velocity.z -= 0.5 * time.delta_secs(); // Gravity
                particle.velocity.x *= 0.98; // Air resistance
                particle.velocity.y *= 0.98;

                map_color.color.set_alpha(life_ratio * 0.6);
                transform.scale = Vec3::splat(0.3 + life_ratio * 0.2);
            }
            InteractionParticleType::Trail => {
                // Trail particles fade quickly and move in original direction
                map_color.color.set_alpha(life_ratio * 0.4);
                transform.scale = Vec3::splat(0.2 + life_ratio * 0.1);
            }
            InteractionParticleType::HauntedGlow => {
                // Haunted glow particles float upward and pulse
                particle.velocity.z += 0.2 * time.delta_secs(); // Float upward
                let pulse = (time.elapsed_secs() * 3.0).sin() * 0.2 + 0.8;
                map_color.color.set_alpha(life_ratio * 0.7 * pulse);
                transform.scale = Vec3::splat(0.4 + pulse * 0.2);
            }
            InteractionParticleType::Spark => {
                // Sparks fall down quickly and flicker
                particle.velocity.z -= 2.0 * time.delta_secs(); // Strong gravity
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
}

/// System that handles door lock visual indicators
fn door_lock_indicator_system(
    mut commands: Commands,
    time: Res<Time>,
    _asset_server: Res<AssetServer>,
    mut q_indicators: Query<(Entity, &mut LockIndicator, &mut Sprite)>,
    q_locked_doors: Query<Entity, Added<Locked>>,
    q_unlocked_doors: Query<Entity, (With<LockIndicator>, Without<Locked>)>,
) {
    // Spawn lock indicators for newly locked doors
    for door_entity in q_locked_doors.iter() {
        commands.entity(door_entity).insert(LockIndicator::new());

        // TODO: Add actual lock icon sprite overlay here
        // For now, we'll modify the door's existing sprite to show it's locked
    }

    // Remove lock indicators from unlocked doors
    for door_entity in q_unlocked_doors.iter() {
        commands.entity(door_entity).remove::<LockIndicator>();
    }

    // Update existing lock indicators
    for (_entity, mut indicator, mut sprite) in q_indicators.iter_mut() {
        indicator.pulse_timer.tick(time.delta());

        // Create pulsing effect
        let pulse_progress = indicator.pulse_timer.fraction();
        let pulse_alpha = (pulse_progress * std::f32::consts::PI * 2.0).sin() * 0.3 + 0.7;

        // Tint the sprite to indicate it's locked (reddish tint)
        sprite.color = Color::srgba(1.0, 0.7, 0.7, indicator.base_alpha * pulse_alpha);
    }
}

/// Helper function to spawn trail particles for thrown objects
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
        .insert(SpriteType::Other)
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

/// Helper function to spawn dust particles for impacts and nudges
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
                global_z: position.global_z,
            })
            .insert(GameSprite)
            .insert(MapColor {
                color: Color::srgba(0.6, 0.5, 0.4, 0.6),
            })
            .insert(SpriteType::Other)
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

/// Helper function to spawn haunted glow particles for creepy movement
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
            global_z: position.global_z,
        })
        .insert(GameSprite)
        .insert(MapColor {
            color: Color::srgba(0.3, 0.8, 0.3, 0.7),
        })
        .insert(SpriteType::Other)
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

/// Helper function to spawn electrical sparks for breaker trips
pub fn spawn_electrical_sparks(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Position,
) {
    let mut rng = random_seed::rng();

    for _ in 0..8 {
        commands
            .spawn(Sprite {
                image: asset_server.load("img/particle_spark.png"),
                color: Color::srgba(1.0, 0.9, 0.3, 0.9),
                custom_size: Some(Vec2::new(0.1, 0.1)),
                ..default()
            })
            .insert(Position {
                x: position.x + rng.random_range(-0.1..0.1),
                y: position.y + rng.random_range(-0.1..0.1),
                z: position.z + rng.random_range(0.1..0.4),
                global_z: position.global_z,
            })
            .insert(GameSprite)
            .insert(MapColor {
                color: Color::srgba(1.0, 0.9, 0.3, 0.9),
            })
            .insert(SpriteType::Other)
            .insert(InteractionParticle {
                life: 0.8,
                max_life: 0.8,
                velocity: Vec3::new(
                    rng.random_range(-0.3..0.3),
                    rng.random_range(-0.3..0.3),
                    rng.random_range(0.2..0.6),
                ),
                particle_type: InteractionParticleType::Spark,
            })
            .insert(Transform::from_scale(Vec3::splat(0.3)));
    }
}
