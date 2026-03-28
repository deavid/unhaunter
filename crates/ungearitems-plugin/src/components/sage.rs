use bevy::prelude::*;
use rand::RngExt;
use unaudiospatial_core::emitter::AudioEmitter;
use unboard_core::components::mapcolor::MapColor;
use uncommon_app_core::random_seed;
use uncommon_app_core::utils::time::format_time;
use ungear_core::components::core::{GearSprite, StatusText};
use ungear_core::types::gear::sprite_id::GearSpriteID;
use ungearitems_core::components::sage::{
    SageBundleData, SageBundleSkin, SageSmokeParticle, SmokeParticleTimer,
};
use unghost_core::components::ghost_sprite::GhostSprite;
use uninteraction_core::interaction::Triggered;
use unmetrics_core::metrics::SendMetric;
use unrender_std::components::game::GameSprite;
use unrender_std::components::sprite_layer::SpriteLayer;
use unreplicon_core::ownership::LocallyOwned;
use unreplicon_core::resources::LocalPlayerRole;
use unspatial_core::direction::Direction;
use unspatial_core::perspective;
use unspatial_core::position::Position;

use crate::metrics;

pub(crate) fn update_sage_skeleton(
    mut q_sage: Query<(Entity, &mut SageBundleData), With<LocallyOwned>>,
    q_triggered: Query<&Triggered>,
    mut commands: Commands,
    mut gs_audio: AudioEmitter,
) {
    for (entity, mut sage) in q_sage.iter_mut() {
        if q_triggered.get(entity).is_ok() && !sage.is_active && !sage.consumed {
            sage.is_active = true;
            commands.entity(entity).insert(SageBundleSkin::new());
            gs_audio.play_audio_nopos("sounds/sage_activation.ogg".into(), 0.8);
            commands.entity(entity).remove::<Triggered>();
        }
    }
}

pub(crate) fn update_sage_skin(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut q_sage: Query<(
        &SageBundleData,
        &mut SageBundleSkin,
        &mut StatusText,
        &mut GearSprite,
        &Position,
    )>,
    time: Res<Time>,
) {
    for (sage, mut skin, mut status, mut sprite, pos) in q_sage.iter_mut() {
        if sage.is_active && !sage.consumed {
            skin.burn_timer.tick(time.delta());

            if skin.burn_timer.just_finished() {
                // Note: We don't modify SageBundleData here as it's a skeleton field.
                // The skeleton system should handle the transition.
                // However, for visual consistency, we can stop smoke.
            } else {
                let target_smoke = (skin.burn_timer.elapsed_secs() * 3.0) as usize;
                if skin.smoke_produced < target_smoke {
                    for _ in skin.smoke_produced..target_smoke {
                        let mut p = *pos;
                        let mut rng = random_seed::rng();
                        p.z += 0.2;
                        p.x += rng.random_range(-0.2..0.2);
                        p.y += rng.random_range(-0.2..0.2);

                        commands
                            .spawn(Sprite {
                                image: asset_server.load("img/smoke.png"),
                                color: Color::NONE,
                                ..default()
                            })
                            .insert(
                                Transform::from_translation(perspective::to_screen_coord(p))
                                    .with_scale(Vec3::new(0.2, 0.2, 0.2)),
                            )
                            .insert(SageSmokeParticle)
                            .insert(GameSprite)
                            .insert(p)
                            .insert(Direction {
                                dx: rng.random_range(-0.9..0.9),
                                dy: rng.random_range(-0.9..0.9),
                                dz: rng.random_range(-0.5..0.5),
                            })
                            .insert(MapColor {
                                color: Color::srgba(1.0, 1.0, 1.0, 0.20),
                            })
                            .insert(SmokeParticleTimer(Timer::from_seconds(
                                5.0,
                                TimerMode::Once,
                            )))
                            .insert(SpriteLayer::default());
                    }
                    skin.smoke_produced = target_smoke;
                }
            }
        }

        if sage.consumed {
            status.0 = "Sage Bundle: Consumed".to_string();
        } else if !sage.is_active {
            status.0 = "Sage Bundle: Ready".to_string();
        } else {
            status.0 = format!(
                "Sage Bundle: Burning: {}",
                format_time(skin.burn_timer.remaining_secs())
            );
        }

        sprite.0 = if sage.consumed {
            GearSpriteID::SageBundle4.to_visual_key()
        } else if !sage.is_active {
            GearSpriteID::SageBundle0.to_visual_key()
        } else {
            let remaining_time = skin.burn_timer.remaining_secs();
            if remaining_time > 5.0 {
                GearSpriteID::SageBundle1.to_visual_key()
            } else if remaining_time > 3.0 {
                GearSpriteID::SageBundle2.to_visual_key()
            } else if remaining_time > 0.0 {
                GearSpriteID::SageBundle3.to_visual_key()
            } else {
                GearSpriteID::SageBundle4.to_visual_key()
            }
        };
    }
}

fn sage_smoke_system(
    mut commands: Commands,
    time: Res<Time>,
    mut smoke_particles: Query<
        (
            Entity,
            &mut Position,
            &mut Transform,
            &mut SmokeParticleTimer,
            &mut MapColor,
            Option<&Direction>,
        ),
        (Without<GhostSprite>, With<SageSmokeParticle>),
    >,
    mut ghosts: Query<(&mut GhostSprite, &Position)>,
) {
    let measure = metrics::SAGE_SMOKE.time_measure();

    let dt = time.delta_secs();
    for (entity, mut position, mut transform, mut smoke_particle, mut map_color, o_dir) in
        smoke_particles.iter_mut()
    {
        smoke_particle.0.tick(time.delta());
        if smoke_particle.0.just_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let dir = o_dir.unwrap_or(&Direction {
            dx: 0.0,
            dy: 0.0,
            dz: 0.0,
        });
        let elap = smoke_particle.0.elapsed_secs();
        let rem = smoke_particle.0.remaining_secs();
        let a = ((elap * 3.0)
            .clamp(0.0, 1.0)
            .min((rem / 2.0 - 0.01).clamp(0.0, 1.0)))
        .powf(2.0);
        map_color.color.set_alpha(a * 0.4);

        position.z += 0.3 * dt / (1.0 + elap.powi(2));
        position.x += dir.dx * dt;
        position.y += dir.dy * dt;
        transform.scale.x += 0.1 * dt;
        transform.scale.y += 0.1 * dt;

        for (mut ghost, ghost_position) in ghosts.iter_mut() {
            let dist = position.distance(ghost_position);
            if dist < 5.0 {
                ghost.rage -= 30.0 * dt * a / (1.0 + dist);
                if ghost.rage < 0.0 {
                    ghost.rage = 0.0;
                }
                ghost.calm_time_secs += 10.0 * dt * a / (1.0 + dist);
                if ghost.calm_time_secs > 30.0 {
                    ghost.calm_time_secs = 30.0;
                }
            }
        }
    }

    measure.end_ms();
}

pub(crate) fn hydrate_sage_skin(
    mut commands: Commands,
    q_added: Query<Entity, (Added<SageBundleData>, Without<SageBundleSkin>)>,
) {
    for entity in q_added.iter() {
        commands.entity(entity).insert(SageBundleSkin::new());
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, (update_sage_skeleton, hydrate_sage_skin));
    app.add_systems(
        Update,
        (update_sage_skin, sage_smoke_system).run_if(resource_exists::<LocalPlayerRole>),
    );
}
