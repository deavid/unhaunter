//! This module defines the `SageBundleData` struct and its associated logic,
//! representing the Sage Bundle consumable item in the game.

use bevy::prelude::*;

use super::{Gear, GearKind, GearSpriteID, GearStuff, GearUsable};
use crate::{board::Position, ghost::GhostSprite, utils::format_time};

/// Data structure for the Sage Bundle consumable.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct SageBundleData {
    /// Indicates whether the Sage Bundle is currently active (burning).
    pub is_active: bool,
    /// Timer for the burn duration.
    pub burn_timer: Timer,
}

impl Default for SageBundleData {
    fn default() -> Self {
        Self {
            is_active: false,
            burn_timer: Timer::from_seconds(6.0, TimerMode::Once),
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
        if !self.is_active {
            return "Ready".to_string();
        }
        format!("Burning: {}", format_time(self.burn_timer.remaining_secs()))
    }

    fn set_trigger(&mut self, gs: &mut GearStuff) {
        if !self.is_active {
            self.is_active = true;
            self.burn_timer.reset();

            // Play activation sound
            gs.play_audio_nopos("sounds/sage_activation.ogg".into(), 0.6);
        }
    }

    fn update(
        &mut self,
        gs: &mut GearStuff,
        pos: &Position,
        _ep: &super::playergear::EquipmentPosition,
    ) {
        if self.is_active {
            self.burn_timer.tick(gs.time.delta());

            // Spawn smoke particles
            if self.burn_timer.just_finished() {
                self.is_active = false;
            } else if gs.time.elapsed_seconds() % 0.3 < 0.1 {
                for _ in 0..3 {
                    // Spawn smoke particle
                    let _smoke_particle_entity = gs
                        .commands
                        .spawn(SpriteBundle {
                            texture: gs.asset_server.load("img/consumables.png"),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(8.0, 8.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(pos.to_screen_coord()),
                            ..default()
                        })
                        .insert(SageSmokeParticle)
                        .insert(*pos)
                        .insert(SmokeParticleTimer(Timer::from_seconds(
                            3.0,
                            TimerMode::Once,
                        )))
                        .id();
                }
            }
        }
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        if !self.is_active {
            return GearSpriteID::SageBundle0;
        }
        let remaining_time = self.burn_timer.remaining_secs();
        if remaining_time > 4.0 {
            return GearSpriteID::SageBundle1;
        }
        if remaining_time > 2.0 {
            return GearSpriteID::SageBundle2;
        }
        if remaining_time > 0.0 {
            return GearSpriteID::SageBundle3;
        }
        GearSpriteID::SageBundle4 // Burned out
    }

    fn _box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<SageBundleData> for Gear {
    fn from(value: SageBundleData) -> Self {
        Gear::new_from_kind(GearKind::SageBundle(value))
    }
}

/// Marker component for sage smoke particles.
#[derive(Component)]
pub struct SageSmokeParticle;

/// Timer component for smoke particle lifetime.
#[derive(Component)]
pub struct SmokeParticleTimer(Timer);

/// System to handle smoke particle logic.
pub fn sage_smoke_system(
    mut commands: Commands,
    time: Res<Time>,
    mut smoke_particles: Query<(Entity, &mut Position, &mut SmokeParticleTimer)>,
    mut ghosts: Query<(&mut GhostSprite, &Position)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut position, mut smoke_particle) in smoke_particles.iter_mut() {
        smoke_particle.0.tick(time.delta());
        if smoke_particle.0.just_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        // Make particles float upwards
        position.z += 0.01 * dt;

        // Apply calming effect to ghost if within range
        for (mut ghost, ghost_position) in ghosts.iter_mut() {
            if position.distance(ghost_position) < 16.0 {
                ghost.rage -= 0.1;
                if ghost.rage < 0.0 {
                    ghost.rage = 0.0;
                }
            }
        }
    }
}
