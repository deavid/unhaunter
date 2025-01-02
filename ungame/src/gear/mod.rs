//! ## Gear Module
//!
//! This module defines the gear system for the game, including:
//!
//! * Different types of gear available to the player.
//!
//! * A common interface for interacting with gear (`GearUsable` trait).
//!
//! * Functions for updating gear state based on player actions and game conditions.
//!
//! * Visual representations of gear using sprites and animations.
//!
//! The gear system allows players to equip and use various tools to investigate
//! paranormal activity, gather evidence, and ultimately banish ghosts.
pub mod ext;
pub mod playergear;
pub mod prelude;
pub mod ui;

use self::playergear::PlayerGear;
use crate::board::Position;
use crate::game::GameConfig;
use crate::player::{DeployedGear, DeployedGearData, PlayerSprite};
use bevy::prelude::*;
use ext::systemparam::gearstuff::GearStuff;
use uncore::events::sound::SoundEvent;

pub use uncore::types::gear::spriteid::GearSpriteID;

use ext::types::traits::GearUsable;

/// System for updating the internal state of all gear carried by the player.
///
/// This system iterates through the player's gear and calls the `update` method
/// for each piece of gear, allowing gear to update their state based on time,
/// player actions, or environmental conditions.
pub fn update_playerheld_gear_data(
    mut q_gear: Query<(&Position, &mut PlayerGear)>,
    mut gs: GearStuff,
) {
    for (position, mut playergear) in q_gear.iter_mut() {
        for (gear, epos) in playergear.as_vec_mut().into_iter() {
            gear.update(&mut gs, position, &epos);
        }
    }
}

/// System for updating the internal state of all gear deployed in the environment.
pub fn update_deployed_gear_data(
    mut q_gear: Query<(&Position, &DeployedGear, &mut DeployedGearData)>,
    mut gs: GearStuff,
) {
    for (position, _deployed_gear, mut gear_data) in q_gear.iter_mut() {
        gear_data
            .gear
            .update(&mut gs, position, &playergear::EquipmentPosition::Deployed);
    }
}

/// System for updating the sprites of deployed gear to reflect their internal
/// state.
pub fn update_deployed_gear_sprites(mut q_gear: Query<(&mut Sprite, &DeployedGearData)>) {
    for (mut sprite, gear_data) in q_gear.iter_mut() {
        let new_index = gear_data.gear.get_sprite_idx() as usize;
        if let Some(ref mut texture_atlas) = &mut sprite.texture_atlas {
            if texture_atlas.index != new_index {
                texture_atlas.index = new_index;
            }
        }
    }
}

/// System to handle the SoundEvent, playing the sound with volume adjusted by
/// distance.
pub fn sound_playback_system(
    mut sound_events: EventReader<SoundEvent>,
    asset_server: Res<AssetServer>,
    gc: Res<GameConfig>,
    qp: Query<(&Position, &PlayerSprite)>,
    mut commands: Commands,
) {
    for sound_event in sound_events.read() {
        // Get player position (Match against the player ID from GameConfig)
        let Some((player_position, _)) = qp.iter().find(|(_, p)| p.id == gc.player_id) else {
            return;
        };
        let adjusted_volume = match sound_event.position {
            Some(position) => {
                const MIN_DIST: f32 = 25.0;

                // Calculate distance from player to sound source
                let distance2 = player_position.distance2(&position) + MIN_DIST;
                let distance = distance2.powf(0.7) + MIN_DIST;

                // Calculate adjusted volume based on distance
                (sound_event.volume / distance2 * MIN_DIST
                    + sound_event.volume / distance * MIN_DIST)
                    .clamp(0.0, 1.0)
            }
            None => sound_event.volume,
        };

        // Spawn an AudioBundle with the adjusted volume
        commands
            .spawn(AudioPlayer::<AudioSource>(
                asset_server.load(sound_event.sound_file.clone()),
            ))
            .insert(PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: bevy::audio::Volume::new(adjusted_volume),
                speed: 1.0,
                paused: false,
                spatial: false,
                spatial_scale: None,
            });
    }
}

pub fn app_setup(app: &mut App) {
    use crate::gear::ext::types::items::*;

    app.init_resource::<GameConfig>()
        .add_systems(FixedUpdate, update_playerheld_gear_data)
        .add_systems(FixedUpdate, update_deployed_gear_data)
        .add_systems(FixedUpdate, update_deployed_gear_sprites)
        .add_systems(Update, quartz::update_quartz_and_ghost)
        .add_systems(Update, salt::salt_particle_system)
        .add_systems(Update, salt::salt_pile_system)
        .add_systems(Update, salt::salty_trace_system)
        .add_systems(Update, sage::sage_smoke_system)
        .add_systems(Update, thermometer::temperature_update)
        .add_systems(Update, recorder::sound_update)
        .add_systems(Update, repellentflask::repellent_update)
        .add_systems(Update, sound_playback_system)
        .add_event::<SoundEvent>();
    ui::app_setup(app);
}
