//! This module defines the `SaltData` struct and its associated logic,
//! representing the Salt consumable item in the game.

use bevy::prelude::*;
use rand::Rng as _;

use crate::{board::Position, ghost::GhostSprite};

use super::{Gear, GearKind, GearSpriteID, GearStuff, GearUsable};

/// Data structure for the Salt consumable.
#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct SaltData {
    /// Number of salt charges remaining (0-4).
    pub charges: u8,
    /// If salt should be spawned on the next frame.
    pub spawn_salt: bool,
}

impl GearUsable for SaltData {
    fn get_display_name(&self) -> &'static str {
        "Salt"
    }

    fn get_description(&self) -> &'static str {
        "A bottle containing four charges of salt. Players can drop salt piles to repel the ghost and create temporary trails of UV-reactive salt particles."
    }

    fn get_status(&self) -> String {
        format!("Charges: {}", self.charges)
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {
        if self.charges > 0 && !self.spawn_salt {
            self.charges -= 1;
            self.spawn_salt = true;
        }
    }

    fn update(
        &mut self,
        gs: &mut GearStuff,
        pos: &Position,
        _ep: &super::playergear::EquipmentPosition,
    ) {
        if self.spawn_salt {
            // Spawn salt pile entity

            self.spawn_salt = false;
            let _salt_pile_entity = gs
                .commands
                .spawn(SpriteBundle {
                    texture: gs.asset_server.load("img/salt_pile.png"),
                    transform: Transform::from_translation(pos.to_screen_coord()),
                    ..default()
                })
                .insert(SaltPile)
                .id();

            gs.play_audio("sounds/salt_drop.ogg".into(), 1.0, pos);
        }
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.charges {
            4 => GearSpriteID::Salt4,
            3 => GearSpriteID::Salt3,
            2 => GearSpriteID::Salt2,
            1 => GearSpriteID::Salt1,
            _ => GearSpriteID::Salt0, // Empty
        }
    }

    fn _box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<SaltData> for Gear {
    fn from(value: SaltData) -> Self {
        Gear::new_from_kind(GearKind::Salt(value))
    }
}

/// Marker component for salt pile entities.
#[derive(Component)]
pub struct SaltPile;

/// Marker component for salt particle entities.
#[derive(Component)]
pub struct SaltParticle;

/// Timer component for salt particle lifetime.
#[derive(Component)]
pub struct SaltParticleTimer(Timer);

/// System to handle salt pile logic.
pub fn salt_pile_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut ghosts: Query<(&mut GhostSprite, &Transform)>,
    salt_piles: Query<(Entity, &Transform), With<SaltPile>>,
) {
    let dt = time.delta_seconds();
    for (mut ghost, ghost_transform) in ghosts.iter_mut() {
        for (salt_pile_entity, salt_pile_transform) in salt_piles.iter() {
            if ghost_transform
                .translation
                .distance(salt_pile_transform.translation)
                < 16.0
            {
                // Increase ghost rage
                ghost.rage += 10.0 * dt;

                // FIXME: This is incorrect. It should instead give a side-effect
                // to the ghost, and then the spawning of these would be in a ghost system
                // The ghost system might be created on this salt.rs file, but it needs to be a separate one.

                // Spawn salt particles
                for _ in 0..5 {
                    let particle_position = salt_pile_transform.translation
                        + Vec3::new(
                            rand::thread_rng().gen_range(-4.0..4.0),
                            rand::thread_rng().gen_range(-4.0..4.0),
                            0.0,
                        );

                    let _salt_particle_entity = commands
                        .spawn(SpriteBundle {
                            texture: asset_server.load("img/salt_particle.png"),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(4.0, 4.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(particle_position),
                            ..default()
                        })
                        .insert(SaltParticle)
                        .insert(SaltParticleTimer(Timer::from_seconds(3.0, TimerMode::Once)))
                        .id();
                }

                // Despawn salt pile
                commands.entity(salt_pile_entity).despawn();
            }
        }
    }
}

/// System to handle salt particle logic.
pub fn salt_particle_system(
    mut commands: Commands,
    time: Res<Time>,
    mut salt_particles: Query<(Entity, &mut Transform, &mut SaltParticleTimer)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut transform, mut salt_particle_timer) in salt_particles.iter_mut() {
        salt_particle_timer.0.tick(time.delta());
        if salt_particle_timer.0.just_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        // Update salt particle position (currently just random movement)
        transform.translation.x += rand::thread_rng().gen_range(-0.1..0.1) * dt;
        transform.translation.y += rand::thread_rng().gen_range(-0.1..0.1) * dt;
        transform.translation.z += rand::thread_rng().gen_range(-0.01..0.01) * dt;

        // Apply fade-out effect
        transform.scale.x /= 1.05;
        transform.scale.y /= 1.05;
        transform.scale.z /= 1.05;
        transform.scale.x = transform.scale.x.max(0.00001);
        transform.scale.y = transform.scale.y.max(0.00001);
        transform.scale.z = transform.scale.z.max(0.00001);
    }
}
