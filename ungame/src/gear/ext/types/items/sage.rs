//! This module defines the `SageBundleData` struct and its associated logic,
//! representing the Sage Bundle consumable item in the game.
use uncore::{
    components::{
        board::{direction::Direction, position::Position},
        game::GameSprite,
    },
    types::gear::equipmentposition::EquipmentPosition,
    utils::format_time,
};

use crate::{ghost::GhostSprite, maplight::MapColor};

use super::{Gear, GearKind, GearSpriteID, GearStuff, GearUsable};

use bevy::prelude::*;
use rand::Rng;

/// Data structure for the Sage Bundle consumable.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct SageBundleData {
    /// Indicates whether the Sage Bundle is currently active (burning).
    pub is_active: bool,
    /// Timer for the burn duration.
    pub burn_timer: Timer,
    /// If the sage has been completely burned.
    pub consumed: bool,
    /// Amount of particles of smoke produced. Used to pace the smoke production.
    pub smoke_produced: usize,
}

impl Default for SageBundleData {
    fn default() -> Self {
        Self {
            is_active: false,
            burn_timer: Timer::from_seconds(8.0, TimerMode::Once),
            consumed: false,
            smoke_produced: 0,
        }
    }
}

impl GearUsable for SageBundleData {
    fn get_display_name(&self) -> &'static str {
        "Sage Bundle"
    }

    fn get_description(&self) -> &'static str {
        "A bundle of sage that, when activated, burns slowly and emits soothing smoke particles that calm the ghost over time."
    }

    fn get_status(&self) -> String {
        if self.consumed {
            return "Consumed".to_string();
        }
        if !self.is_active {
            return "Ready".to_string();
        }
        format!("Burning: {}", format_time(self.burn_timer.remaining_secs()))
    }

    fn set_trigger(&mut self, gs: &mut GearStuff) {
        if !self.is_active && !self.consumed {
            self.is_active = true;
            self.burn_timer.reset();

            // Play activation sound
            gs.play_audio_nopos("sounds/sage_activation.ogg".into(), 0.8);
        }
    }

    fn update(&mut self, gs: &mut GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        if self.is_active && !self.consumed {
            self.burn_timer.tick(gs.time.delta());

            // Spawn smoke particles
            if self.burn_timer.just_finished() {
                self.is_active = false;
                self.consumed = true;
            } else if (self.smoke_produced as f32) < self.burn_timer.elapsed_secs() * 3.0 {
                let mut pos = *pos;
                let mut rng = rand::thread_rng();
                pos.z += 0.2;
                pos.x += rng.gen_range(-0.2..0.2);
                pos.y += rng.gen_range(-0.2..0.2);

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
                    )));
                self.smoke_produced += 1;
            }
        }
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        if self.consumed {
            // Burned out
            return GearSpriteID::SageBundle4;
        }
        if !self.is_active {
            return GearSpriteID::SageBundle0;
        }
        let remaining_time = self.burn_timer.remaining_secs();
        if remaining_time > 5.0 {
            return GearSpriteID::SageBundle1;
        }
        if remaining_time > 3.0 {
            return GearSpriteID::SageBundle2;
        }
        if remaining_time > 0.0 {
            return GearSpriteID::SageBundle3;
        }

        // Burned out
        GearSpriteID::SageBundle4
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<SageBundleData> for Gear {
    fn from(value: SageBundleData) -> Self {
        Gear::new_from_kind(GearKind::SageBundle(value.box_clone()))
    }
}

/// Marker component for sage smoke particles.
#[derive(Component)]
pub struct SageSmokeParticle;

/// Timer component for smoke particle lifetime.
#[derive(Component)]
pub struct SmokeParticleTimer(pub Timer);

/// System to handle smoke particle logic.
#[allow(clippy::type_complexity)]
pub fn sage_smoke_system(
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
}
