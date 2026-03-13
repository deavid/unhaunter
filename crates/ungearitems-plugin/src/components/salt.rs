use bevy::prelude::*;
use rand::RngExt;
use unboard_core::components::mapcolor::MapColor;
use unfoundation_core::random_seed;
use ungear_core::components::core::{GearSprite, StatusText};
use ungear_core::types::gear::sprite_id::GearSpriteID;
use ungearitems_core::components::salt::{
    SaltData, SaltParticle, SaltParticleTimer, SaltPile, SaltyTrace, SaltyTraceTimer, UVReactive,
};
use unghost_core::components::ghost_sprite::GhostSprite;
use uninteraction_core::interaction::Triggered;
use unmetrics_core::metrics::SendMetric;
use unrender_std::components::game::GameSprite;
use unrender_std::components::sprite_layer::SpriteLayer;
use unreplicon_core::ownership::LocallyOwned;
use unsound_core::emitter::SoundEmitter;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use untypes_core::roles::LocalPlayerRole;

use crate::metrics;

pub(crate) fn update_salt_skeleton(
    mut q_salt: Query<(Entity, &mut SaltData, &Position), With<LocallyOwned>>,
    q_triggered: Query<&Triggered>,
    mut gs_audio: SoundEmitter,
    mut commands: Commands,
) {
    for (entity, mut salt, pos) in q_salt.iter_mut() {
        if q_triggered.get(entity).is_ok() && salt.charges > 0 {
            salt.charges -= 1;

            commands
                .spawn(Sprite {
                    image: gs_audio.asset_server.load("img/salt_pile.png"),
                    ..default()
                })
                .insert(
                    Transform::from_translation(perspective::to_screen_coord(*pos))
                        .with_scale(Vec3::new(0.5, 0.5, 0.5)),
                )
                .insert(SaltPile)
                .insert(GameSprite)
                .insert(*pos)
                .insert(SpriteLayer::default());
            gs_audio.play_audio("sounds/salt_drop.ogg".into(), 1.0, pos);

            commands.entity(entity).remove::<Triggered>();
        }
    }
}

pub(crate) fn update_salt_skin(mut q_salt: Query<(&SaltData, &mut StatusText, &mut GearSprite)>) {
    for (salt, mut status, mut sprite) in q_salt.iter_mut() {
        status.0 = format!("Charges: {}", salt.charges);
        sprite.0 = match salt.charges {
            4 => GearSpriteID::Salt4.to_visual_key(),
            3 => GearSpriteID::Salt3.to_visual_key(),
            2 => GearSpriteID::Salt2.to_visual_key(),
            1 => GearSpriteID::Salt1.to_visual_key(),
            _ => GearSpriteID::Salt0.to_visual_key(),
        };
    }
}

fn salt_pile_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ghosts: Query<(&mut GhostSprite, &Position)>,
    mut salt_piles: Query<(Entity, &Position), With<SaltPile>>,
) {
    let measure = metrics::SALT_PILE.time_measure();

    for (mut ghost, ghost_position) in ghosts.iter_mut() {
        for (salt_pile_entity, salt_pile_position) in salt_piles.iter_mut() {
            if ghost_position.distance(salt_pile_position) < 2.0
                && ghost.salty_effect_timer.elapsed_secs() > 1.0
            {
                ghost.rage += 10.0;
                ghost.salty_effect_timer.reset();

                for _ in 0..5 {
                    let mut particle_position = *salt_pile_position;
                    particle_position.x += random_seed::rng().random_range(-0.2..0.2);
                    particle_position.y += random_seed::rng().random_range(-0.2..0.2);
                    commands
                        .spawn(Sprite {
                            image: asset_server.load("img/salt_particle.png"),
                            custom_size: Some(Vec2::new(4.0, 4.0)),
                            ..default()
                        })
                        .insert(Transform::from_translation(perspective::to_screen_coord(
                            particle_position,
                        )))
                        .insert(particle_position)
                        .insert(GameSprite)
                        .insert(SaltParticle)
                        .insert(SaltParticleTimer(Timer::from_seconds(30.0, TimerMode::Once)))
                        .insert(SpriteLayer::default());
                }

                commands.entity(salt_pile_entity).despawn();
            }
        }
    }

    measure.end_ms();
}

fn salt_particle_system(
    mut commands: Commands,
    time: Res<Time>,
    mut salt_particles: Query<(Entity, &mut Transform, &mut SaltParticleTimer)>,
) {
    let measure = metrics::SALT_PARTICLE.time_measure();

    let dt = time.delta_secs();
    for (entity, mut transform, mut salt_particle_timer) in salt_particles.iter_mut() {
        salt_particle_timer.0.tick(time.delta());
        if salt_particle_timer.0.just_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        transform.scale.x /= 1.05_f32.powf(dt);
        transform.scale.y /= 1.05_f32.powf(dt);
        transform.scale.z /= 1.05_f32.powf(dt);
        transform.scale.x = transform.scale.x.max(0.00001);
        transform.scale.y = transform.scale.y.max(0.00001);
        transform.scale.z = transform.scale.z.max(0.00001);
    }
    measure.end_ms();
}

fn salty_trace_system(
    mut commands: Commands,
    time: Res<Time>,
    mut salty_traces: Query<
        (Entity, &mut MapColor, &mut UVReactive, &mut SaltyTraceTimer),
        With<SaltyTrace>,
    >,
) {
    let measure = metrics::SALTY_TRACE.time_measure();

    for (entity, mut map_color, mut uv_reactive, mut salty_trace_timer) in salty_traces.iter_mut() {
        salty_trace_timer.0.tick(time.delta());

        const UV_FADE_DURATION: f32 = 180.0;
        uv_reactive.0 =
            (2.0 - salty_trace_timer.0.elapsed_secs() / UV_FADE_DURATION).clamp(0.0, 1.0);

        const OPACITY_FADE_START: f32 = UV_FADE_DURATION;
        const OPACITY_FADE_DURATION: f32 = 300.0;
        if salty_trace_timer.0.elapsed_secs() > OPACITY_FADE_START {
            let fade_progress =
                (salty_trace_timer.0.elapsed_secs() - OPACITY_FADE_START) / OPACITY_FADE_DURATION;
            map_color.color.set_alpha(1.0 - fade_progress);
        }

        if salty_trace_timer.0.is_finished() && map_color.color.alpha() == 0.0 {
            commands.entity(entity).despawn();
        }
    }

    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_salt_skeleton);
    app.add_systems(
        Update,
        (
            update_salt_skin,
            salt_particle_system,
            salt_pile_system,
            salty_trace_system,
        )
            .run_if(resource_exists::<LocalPlayerRole>),
    );
}
