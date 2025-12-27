use super::components::deployedgear::DeployedGear;
use super::components::playergear::PlayerGear;
use crate::Hand;
use crate::gear_stuff::GearStuff;
use crate::resources::looking_gear::LookingGear;
use crate::resources::spawner::GearSpawnerRegistry;
use bevy::audio::SpatialScale;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore_assets::GameAssets;
use uncore_board::components::mapcolor::MapColor;
use uncore_components::{GearSprite, StatusText, Toggleable, Triggered};
use uncore_events::events::sound::SoundEvent;
use uncore_foundation::types::gear::{GearKind, GearSpriteID};
use uncore_resources::states::GameState;
use unplayer_core::components::{Inventory, InventoryNext, InventoryStats};
use unplayer_core::resources::PlayerState;
use unrender::components::game::GameSprite;
use unrender::components::sprite_type::SpriteType;
use unsettings::audio::{AudioSettings, SoundOutput};
use unspatial::Position;
use untags::PlayerTag;

fn update_deployed_gear_sprites(
    mut commands: Commands,
    mut q_gear: Query<(Entity, &Position, &GearSprite, Option<&mut Sprite>), With<DeployedGear>>,
    handles: Res<GameAssets>,
) {
    for (entity, pos, gear_sprite, sprite) in q_gear.iter_mut() {
        if let Some(mut sprite) = sprite {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = gear_sprite.0 as usize;
            }
        } else {
            commands.entity(entity).insert((
                Sprite {
                    image: handles.images.gear.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: handles.images.gear_atlas.clone(),
                        index: gear_sprite.0 as usize,
                    }),
                    ..default()
                },
                Transform::from_translation(pos.to_screen_coord()).with_scale(Vec3::splat(0.25)),
                Visibility::Inherited,
                GameSprite,
                SpriteType::default(),
                MapColor::default(),
            ));
        }
    }
}

fn sound_playback_system(
    mut sound_events: MessageReader<SoundEvent>,
    asset_server: Res<AssetServer>,
    qp: Query<&Position, With<PlayerTag>>,
    mut commands: Commands,
    audio_settings: Res<Persistent<AudioSettings>>,
) {
    for sound_event in sound_events.read() {
        let Some(player_position) = qp.iter().next() else {
            return;
        };
        if !player_position.is_finite() {
            warn!("Player position is not finite: {player_position:?}")
        }
        let dist = sound_event
            .position
            .map(|pos| player_position.distance(&pos))
            .unwrap_or(0.0);
        let mut adjusted_volume = (sound_event.volume * (1.0 + dist * 0.2)).clamp(0.0, 1.0);
        if audio_settings.sound_output == SoundOutput::Mono {
            adjusted_volume /= 1.0 + dist * 0.4;
        }

        let mut sound = commands.spawn(AudioPlayer::<AudioSource>(
            asset_server.load(sound_event.sound_file.clone()),
        ));
        sound.insert(PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::Linear(
                adjusted_volume
                    * audio_settings.volume_effects.as_f32()
                    * audio_settings.volume_master.as_f32(),
            ),
            speed: 1.0,
            paused: false,
            spatial: sound_event.position.is_some()
                && audio_settings.sound_output != SoundOutput::Mono,
            spatial_scale: Some(SpatialScale::new(0.005)),
            ..default()
        });
        if let Some(position) = sound_event.position {
            let mut spos_vec = position.to_screen_coord();
            spos_vec.z -= 10.0 / audio_settings.sound_output.to_ear_offset();
            sound.insert(Transform::from_translation(spos_vec));
        }
    }
}

fn keyboard_gear(
    _keyboard_input: Res<ButtonInput<KeyCode>>,
    mut _q_gear: Query<&mut PlayerGear, With<PlayerTag>>,
    _player_state: Res<PlayerState>,
    _looking_gear: Res<LookingGear>,
    mut _gs: GearStuff,
) {
    // TODO: Implement using Entity-based gear
}

fn update_gear_ui(
    q_gear: Query<&PlayerGear, With<PlayerTag>>,
    mut qi: Query<(&Inventory, &mut ImageNode), Without<InventoryNext>>,
    mut qin: Query<(&InventoryNext, &mut ImageNode), Without<Inventory>>,
    mut qs: Query<(&InventoryStats, &mut Text, &mut Node)>,
    q_gearkind: Query<&GearKind>,
    q_status: Query<&StatusText>,
    q_sprite: Query<&GearSprite>,
    gear_registry: Res<GearSpawnerRegistry>,
    looking_gear: Res<LookingGear>,
) {
    let Some(player_gear) = q_gear.iter().next() else {
        return;
    };

    for (inv, mut image) in qi.iter_mut() {
        let entity = match inv.hand {
            Hand::Left => player_gear.left_hand,
            Hand::Right => player_gear.right_hand,
        };
        let kind = entity
            .and_then(|e| q_gearkind.get(e).ok())
            .unwrap_or(&GearKind::None);

        let sprite_idx = entity
            .and_then(|e| q_sprite.get(e).ok())
            .map(|s| s.0 as usize)
            .or_else(|| {
                gear_registry
                    .metadata
                    .get(kind)
                    .map(|m| m.sprite_idx as usize)
            })
            .unwrap_or(GearSpriteID::None as usize);

        if let Some(atlas) = &mut image.texture_atlas {
            atlas.index = sprite_idx;
        }
    }

    for (inv_next, mut image) in qin.iter_mut() {
        let entity = inv_next.idx.and_then(|idx| player_gear.inventory.get(idx));
        let kind = entity
            .and_then(|e| q_gearkind.get(*e).ok())
            .unwrap_or(&GearKind::None);

        let sprite_idx = entity
            .and_then(|e| q_sprite.get(*e).ok())
            .map(|s| s.0 as usize)
            .or_else(|| {
                gear_registry
                    .metadata
                    .get(kind)
                    .map(|m| m.sprite_idx as usize)
            })
            .unwrap_or(GearSpriteID::None as usize);

        if let Some(atlas) = &mut image.texture_atlas {
            atlas.index = sprite_idx;
        }
    }

    for (stats, mut text, mut node) in qs.iter_mut() {
        let entity = match stats.hand {
            Hand::Left => player_gear.left_hand,
            Hand::Right => player_gear.right_hand,
        };
        let status = entity
            .and_then(|e| q_status.get(e).ok())
            .map(|s| s.0.clone())
            .unwrap_or_default();
        text.0 = status;

        let is_visible = stats.hand == looking_gear.hand() && !text.0.is_empty();
        node.display = if is_visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn gear_trigger_handler(mut q_toggleable: Query<&mut Toggleable, With<Triggered>>) {
    for mut toggle in q_toggleable.iter_mut() {
        toggle.is_on = !toggle.is_on;
    }
}

fn clear_trigger_handler(mut commands: Commands, q_triggered: Query<Entity, With<Triggered>>) {
    for entity in q_triggered.iter() {
        commands.entity(entity).remove::<Triggered>();
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(FixedUpdate, update_gear_ui)
        .add_systems(Update, update_deployed_gear_sprites)
        .add_systems(Update, keyboard_gear.run_if(in_state(GameState::None)))
        .add_systems(Update, sound_playback_system)
        .add_systems(Update, gear_trigger_handler)
        .add_systems(PostUpdate, clear_trigger_handler);
}
