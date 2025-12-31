use crate::metrics;

use bevy::prelude::*;
use rand::Rng;
use unboard_core::components::mapcolor::MapColor;
use unfoundation_core::random_seed;
use unfoundation_core::types::gear::{EquipmentPosition, GearSpriteID};
use unfoundation_core::utils::time::format_time;
use ungear_core::components::core::{GearSprite, StatusText};
use ungear_core::gear_stuff::GearStuff;
pub use ungearitems_core::components::sage::{
    SageBundleData, SageSmokeParticle, SmokeParticleTimer,
};
use unghost_core::components::ghost_sprite::GhostSprite;
use uninteraction_core::interaction::Triggered;
use unmetrics_core::metrics::SendMetric;
use unrender_std::components::game::GameSprite;
use unrender_std::components::sprite_type::SpriteType;
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;

pub fn update_sage(
    mut gs: GearStuff,
    mut q_sage: Query<(
        Entity,
        &mut SageBundleData,
        &mut StatusText,
        &mut GearSprite,
        &Position,
        &EquipmentPosition,
        Option<&Triggered>,
    )>,
) {
    for (entity, mut sage, mut status, mut sprite, pos, _ep, triggered) in q_sage.iter_mut() {
        if triggered.is_some() && !sage.is_active && !sage.consumed {
            sage.is_active = true;
            sage.burn_timer.reset();

            // Play activation sound
            gs.play_audio_nopos("sounds/sage_activation.ogg".into(), 0.8);

            gs.commands.entity(entity).remove::<Triggered>();
        }

        if sage.is_active && !sage.consumed {
            sage.burn_timer.tick(gs.time.delta());

            // Spawn smoke particles
            if sage.burn_timer.just_finished() {
                sage.is_active = false;
                sage.consumed = true;
            } else if (sage.smoke_produced as f32) < sage.burn_timer.elapsed_secs() * 3.0 {
                let mut pos = *pos;
                let mut rng = random_seed::rng();
                pos.z += 0.2;
                pos.x += rng.random_range(-0.2..0.2);
                pos.y += rng.random_range(-0.2..0.2);

                // Spawn smoke particle
                gs.commands
                    .spawn(Sprite {
                        image: gs.asset_server.load("img/smoke.png"),
                        ..default()
                    })
                    .insert(
                        Transform::from_translation(pos.to_screen_coord())
                            .with_scale(Vec3::new(0.2, 0.2, 0.2)),
                    )
                    .insert(SageSmokeParticle)
                    .insert(GameSprite)
                    .insert(pos)
                    .insert(MapColor {
                        color: Color::WHITE.with_alpha(0.00),
                    })
                    .insert(SmokeParticleTimer(Timer::from_seconds(
                        5.0,
                        TimerMode::Once,
                    )))
                    .insert(SpriteType::Other);
                sage.smoke_produced += 1;
            }
        }

        // Update StatusText
        if sage.consumed {
            status.0 = "Sage Bundle: Consumed".to_string();
        } else if !sage.is_active {
            status.0 = "Sage Bundle: Ready".to_string();
        } else {
            status.0 = format!(
                "Sage Bundle: Burning: {}",
                format_time(sage.burn_timer.remaining_secs())
            );
        }

        // Update GearSprite
        sprite.0 = if sage.consumed {
            GearSpriteID::SageBundle4
        } else if !sage.is_active {
            GearSpriteID::SageBundle0
        } else {
            let remaining_time = sage.burn_timer.remaining_secs();
            if remaining_time > 5.0 {
                GearSpriteID::SageBundle1
            } else if remaining_time > 3.0 {
                GearSpriteID::SageBundle2
            } else if remaining_time > 0.0 {
                GearSpriteID::SageBundle3
            } else {
                GearSpriteID::SageBundle4
            }
        };
    }
}

/// System to handle smoke particle logic.
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

        // Make particles float upwards
        position.z += 0.3 * dt / (1.0 + elap.powi(2));
        position.x += dir.dx * dt;
        position.y += dir.dy * dt;
        transform.scale.x += 0.1 * dt;
        transform.scale.y += 0.1 * dt;

        // Apply calming effect to ghost if within range
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

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_sage);
    app.add_systems(Update, sage_smoke_system);
}
