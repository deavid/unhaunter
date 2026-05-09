use crate::pools::BGAudioPool;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use bevy_seedling::prelude::*;
use unaudiobg_core::smooth::smooth_volume;
use uncommon_states_core::UIContextState;
use unsettings_core::audio::AudioSettings;

/// Component marking an entity as playing menu music.
#[derive(Component, Debug, Default)]
pub(crate) struct MenuSound {
    despawn: bool,
}

/// Unified perceptual smoothing rate for both menu music and background tracks (2.0 units/s)
const VOLUME_SPEED: f32 = 2.0;

/// Manages menu music spawning and lifecycle based on AppState.
/// Spawns menu music when not in game, triggers despawn when entering game.
pub(crate) fn spawn_title_song(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut q_sound: Query<&mut MenuSound>,
    app_state: Res<State<UIContextState>>,
    audio_settings: Res<Persistent<AudioSettings>>,
) {
    let should_play_song = !matches!(app_state.get(), UIContextState::InGame);
    if let Ok(mut menusound) = q_sound.single_mut() {
        if !should_play_song && !menusound.despawn {
            menusound.despawn = true;
        } else if should_play_song && menusound.despawn {
            menusound.despawn = false;
        }
    } else if should_play_song {
        // Only spawn the song if the volume is greater than 0
        let desired_volume =
            audio_settings.volume_music.as_f32() * audio_settings.volume_master.as_f32();
        if desired_volume > 0.0 {
            debug!("spawn_title_song - spawn at: {desired_volume} volume");

            commands.spawn((
                MenuSound::default(),
                SamplePlayer::new(asset_server.load("music/unhaunter_intro.ogg")).looping(),
                sample_effects![VolumeNode {
                    volume: Volume::Linear(desired_volume),
                    ..default()
                }],
                BGAudioPool,
            ));
        }
    }
}

/// Manages menu music volume transitions using unified dB smoothing.
/// Fades out when MenuSound.despawn is true, fades in otherwise.
pub(crate) fn manage_title_song_volume(
    mut commands: Commands,
    q_parents: Query<(Entity, &MenuSound, &SampleEffects)>,
    mut q_volume: Query<&mut VolumeNode>,
    audio_settings: Res<Persistent<AudioSettings>>,
    time: Res<Time>,
) {
    for (entity, menusound, effects) in &q_parents {
        let target_linear = if menusound.despawn {
            0.0
        } else {
            audio_settings.volume_music.as_f32() * audio_settings.volume_master.as_f32()
        };

        if let Ok(mut volume_node) = q_volume.get_effect_mut(effects) {
            let current_linear = volume_node.volume.linear();
            let new_volume = smooth_volume(
                current_linear,
                target_linear,
                VOLUME_SPEED,
                time.delta_secs(),
            );
            volume_node.volume = Volume::Linear(new_volume);
            if new_volume < 0.001 && menusound.despawn {
                commands.entity(entity).despawn();
                debug!("Menu song despawned");
            }
        }
    }
}
