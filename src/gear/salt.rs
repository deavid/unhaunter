//! This module defines the `SaltData` struct and its associated logic,
//! representing the Salt consumable item in the game.
use super::{Gear, GearKind, GearSpriteID, GearStuff, GearUsable};
use crate::{board::Position, game::GameSprite, ghost::GhostSprite, maplight::MapColor};
use bevy::prelude::*;
use rand::Rng as _;

/// Data structure for the Salt consumable.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct SaltData {
    /// Number of salt charges remaining (0-4).
    pub charges: u8,
    /// If salt should be spawned on the next frame.
    pub spawn_salt: bool,
}

impl Default for SaltData {
    fn default() -> Self {
        Self {
            charges: 4,
            spawn_salt: false,
        }
    }
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
            self.spawn_salt = false;

            // Spawn salt pile entity
            let _salt_pile_entity = gs
                .commands
                .spawn(SpriteBundle {
                    texture: gs.asset_server.load("img/salt_pile.png"),
                    transform: Transform::from_translation(pos.to_screen_coord())
                        .with_scale(Vec3::new(0.5, 0.5, 0.5)),
                    ..default()
                })
                .insert(SaltPile)
                .insert(GameSprite)
                .insert(*pos)
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
            // Empty
            _ => GearSpriteID::Salt0,
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
    asset_server: Res<AssetServer>,
    // Retrieve Ghost Position
    mut ghosts: Query<(&mut GhostSprite, &Position)>,
    // Retrieve SaltPile Position
    mut salt_piles: Query<(Entity, &Position), With<SaltPile>>,
) {
    for (mut ghost, ghost_position) in ghosts.iter_mut() {
        for (salt_pile_entity, salt_pile_position) in salt_piles.iter_mut() {
            if ghost_position.distance(salt_pile_position) < 2.0
                && ghost.salty_effect_timer.elapsed_secs() > 1.0
            {
                // Increase ghost rage
                ghost.rage += 10.0;

                // Reset salty_effect_timer to apply the side effect
                ghost.salty_effect_timer.reset();

                // Spawn salt particles
                for _ in 0..5 {
                    // Copy the salt pile's position
                    let mut particle_position = *salt_pile_position;

                    // Add a random offset to the particle position
                    particle_position.x += rand::thread_rng().gen_range(-0.2..0.2);
                    particle_position.y += rand::thread_rng().gen_range(-0.2..0.2);
                    let _salt_particle_entity = commands
                        .spawn(SpriteBundle {
                            texture: asset_server.load("img/salt_particle.png"),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(4.0, 4.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(
                                particle_position.to_screen_coord(),
                            ),
                            ..default()
                        })
                        // Insert the modified Position
                        .insert(particle_position)
                        .insert(GameSprite)
                        .insert(SaltParticle)
                        .insert(SaltParticleTimer(Timer::from_seconds(
                            30.0,
                            TimerMode::Once,
                        )))
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

        // Apply fade-out effect
        transform.scale.x /= 1.05_f32.powf(dt);
        transform.scale.y /= 1.05_f32.powf(dt);
        transform.scale.z /= 1.05_f32.powf(dt);
        transform.scale.x = transform.scale.x.max(0.00001);
        transform.scale.y = transform.scale.y.max(0.00001);
        transform.scale.z = transform.scale.z.max(0.00001);
    }
}

/// Marker component for salt trace entities.
#[derive(Component)]
pub struct SaltyTrace;

/// Component to store the intensity of the green UV glow for SaltyTrace entities.
#[derive(Component)]
pub struct UVReactive(pub f32);

/// Timer component to track the lifetime of a SaltyTrace entity.
#[derive(Component)]
pub struct SaltyTraceTimer(pub Timer);

/// System to handle salt trace logic.
pub fn salty_trace_system(
    mut commands: Commands,
    time: Res<Time>,
    mut salty_traces: Query<
        (Entity, &mut MapColor, &mut UVReactive, &mut SaltyTraceTimer),
        With<SaltyTrace>,
    >,
) {
    for (entity, mut map_color, mut uv_reactive, mut salty_trace_timer) in salty_traces.iter_mut() {
        salty_trace_timer.0.tick(time.delta());

        // --- UV Reactivity Fading --- 3 minutes in seconds
        const UV_FADE_DURATION: f32 = 180.0;
        uv_reactive.0 =
            (2.0 - salty_trace_timer.0.elapsed_secs() / UV_FADE_DURATION).clamp(0.0, 1.0);

        // --- Opacity Fading --- Start fading opacity after UV glow fades
        const OPACITY_FADE_START: f32 = UV_FADE_DURATION;

        // 5 minutes in seconds
        const OPACITY_FADE_DURATION: f32 = 300.0;
        if salty_trace_timer.0.elapsed_secs() > OPACITY_FADE_START {
            let fade_progress =
                (salty_trace_timer.0.elapsed_secs() - OPACITY_FADE_START) / OPACITY_FADE_DURATION;

            // Linear fade
            map_color.color.set_alpha(1.0 - fade_progress);
        }

        // --- Despawn ---
        if salty_trace_timer.0.finished() && map_color.color.alpha() == 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
