use bevy::prelude::*;
use rand::RngExt;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::entity::GameSprite;
use uncommon_app_core::random_seed;
use uncommon_app_core::utils::time::format_time;
use ungear_core::components::core::{GearSprite, StatusText};
use ungear_core::types::gear::sprite_id::GearSpriteID;
use ungearitems_core::components::sage::{
    SageBundleData, SageBundleSkin, SageSmokeParticle, SmokeParticleTimer,
};
use unghost_core::components::logic::ghost_sprite::GhostSprite;
use unmetrics_core::metrics::SendMetric;
use unrender_std::components::sprite_layer::SpriteLayer;
use unreplicon_core::resources::LocalPlayerRole;
use unspatial_core::direction::Direction;
use unspatial_core::perspective;
use unspatial_core::position::Position;

use crate::metrics;

const SAGE_SMOKE_PARTICLES_PER_SECOND: f32 = 30.0;
const SAGE_SMOKE_LIFETIME_SECS: f32 = 5.0;
const SAGE_SMOKE_MAX_ALPHA: f32 = 0.4;
const SAGE_SMOKE_XY_SPREAD: f32 = 0.4;

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
        if sage.is_active && !sage.consumed && !skin.burn_timer.is_finished() {
            let dt = time.delta_secs();
            skin.burn_timer.tick(time.delta());

            if skin.burn_timer.is_finished() {
                // Note: We don't modify SageBundleData here as it's a skeleton field.
                // The skeleton system should handle the transition.
                // However, for visual consistency, we can stop smoke.
            } else {
                skin.smoke_spawn_progress += dt * SAGE_SMOKE_PARTICLES_PER_SECOND;
                let particles_to_spawn = skin.smoke_spawn_progress.floor() as usize;
                if particles_to_spawn > 0 {
                    skin.smoke_spawn_progress -= particles_to_spawn as f32;
                    for _ in 0..particles_to_spawn {
                        let mut p = *pos;
                        let mut rng = random_seed::rng();
                        p.z += 0.2;
                        p.x += rng.random_range(-0.4..0.4);
                        p.y += rng.random_range(-0.4..0.4);

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
                                dx: rng.random_range(-SAGE_SMOKE_XY_SPREAD..SAGE_SMOKE_XY_SPREAD),
                                dy: rng.random_range(-SAGE_SMOKE_XY_SPREAD..SAGE_SMOKE_XY_SPREAD),
                                dz: rng.random_range(-0.05..0.05),
                            })
                            .insert(MapColor {
                                color: Color::srgba(1.0, 1.0, 1.0, SAGE_SMOKE_MAX_ALPHA),
                            })
                            .insert(SmokeParticleTimer(Timer::from_seconds(
                                SAGE_SMOKE_LIFETIME_SECS,
                                TimerMode::Once,
                            )))
                            .insert(SpriteLayer::default());
                    }
                    skin.smoke_produced += particles_to_spawn;
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
    ghosts: Query<(&GhostSprite, &Position)>,
    authority: Option<Res<unreplicon_core::resources::AuthorityRole>>,
    mut ev_sage_hit: MessageWriter<ungearitems_core::events::SageHitNetMessage>,
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
        let fade_in = (elap * 10.0).clamp(0.0, 1.0);
        let fade_out = (rem / 3.5).clamp(0.0, 1.0);
        let a = fade_in.min(fade_out);

        map_color.color.set_alpha(a * SAGE_SMOKE_MAX_ALPHA);

        // Cap Z relative to the starting floor level (integer part of Z) to prevent popping into the next vertical layer:
        let floor_z = position.z.floor();
        let next_z = position.z + (0.3 * dt / (1.0 + elap.powi(2)));
        position.z = next_z.min(floor_z + 0.49);

        position.x += dir.dx * dt;
        position.y += dir.dy * dt;
        transform.scale.x += 0.1 * dt;
        transform.scale.y += 0.1 * dt;

        let mut cumulative_calm = 0.0;
        let mut cumulative_rage_reduction = 0.0;

        for (_ghost, ghost_position) in ghosts.iter() {
            let dist = position.distance(ghost_position);
            if dist < 5.0 {
                let calm = 10.0 * dt * a / (1.0 + dist);
                let rage_reduction = 30.0 * dt * a / (1.0 + dist);

                if authority.is_none() {
                    cumulative_calm += calm;
                    cumulative_rage_reduction += rage_reduction;
                }
            }
        }

        if cumulative_calm > 0.0 || cumulative_rage_reduction > 0.0 {
            ev_sage_hit.write(ungearitems_core::events::SageHitNetMessage {
                calm_this_frame: cumulative_calm,
                rage_reduction_this_frame: cumulative_rage_reduction,
            });
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
    app.add_systems(
        Update,
        (
            hydrate_sage_skin,
            update_sage_skin,
            sage_smoke_system,
            play_sage_effects_audio,
        )
            .run_if(resource_exists::<LocalPlayerRole>),
    );
}

#[derive(Component, Default)]
struct SageAudioPlayed;

fn play_sage_effects_audio(
    mut commands: Commands,
    q_sage: Query<(Entity, &SageBundleData), Without<SageAudioPlayed>>,
    mut gs_audio: unaudiospatial_core::emitter::LocalAudioEmitter,
) {
    for (entity, sage) in q_sage.iter() {
        if sage.is_active && !sage.consumed {
            gs_audio.play_audio_nopos("sounds/sage_activation.ogg".into(), 0.8);
            commands.entity(entity).insert(SageAudioPlayed);
        }
    }
}
