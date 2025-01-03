use super::components::deployedgear::{DeployedGear, DeployedGearData};
use super::components::playergear::PlayerGear;
use bevy::prelude::*;
use uncore::components::board::position::Position;
use uncore::components::game_config::GameConfig;
use uncore::components::player_sprite::PlayerSprite;
use uncore::events::sound::SoundEvent;
use uncore::systemparam::gear_stuff::GearStuff;
use uncore::traits::gear_usable::GearUsable;
use uncore::types::gear::equipmentposition::EquipmentPosition;

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
            .update(&mut gs, position, &EquipmentPosition::Deployed);
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
