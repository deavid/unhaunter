use bevy::prelude::*;
use bevy_persistent::Persistent;
use bevy_seedling::prelude::*;
use uncommon_states_core::UIContextState;
use unsettings_core::audio::AudioSettings;

use unaudiobg_core::smooth::smooth_volume;

/// Component marking an entity as playing menu music.
#[derive(Component, Debug, Default)]
pub(crate) struct MenuSound {
    despawn: bool,
}

/// Unified perceptual smoothing rate for both menu music and background tracks (2.0 units/s)
const VOLUME_SPEED: f32 = 2.0;

/// Manages menu music spawning and lifecycle based on AppState.
/// Spawns menu music when not in game, triggers despawn when entering game.
pub(crate) fn manage_title_song(
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
        let desired_volume = audio_settings.volume_music.as_f32()
            * audio_settings.volume_master.as_f32();
        if desired_volume > 0.0 {
            commands.spawn((
                MenuSound::default(),
                SamplePlayer::new(asset_server.load("music/unhaunter_intro.ogg")).looping(),
                sample_effects![VolumeNode {
                    volume: Volume::Linear(desired_volume),
                    ..default()
                }],
                MusicPool,
            ));
        }
    }
}

/// Manages menu music volume transitions using unified dB smoothing.
/// Fades out when MenuSound.despawn is true, fades in otherwise.
pub(crate) fn despawn_sound(
    mut commands: Commands,
    mut qs: Query<(Entity, &mut VolumeNode, &MenuSound)>,
    audio_settings: Res<Persistent<AudioSettings>>,
    time: Res<Time>,
) {
    for (entity, mut volume_node, menusound) in &mut qs {
        let current_linear = volume_node.volume.linear();

        let target_linear = if menusound.despawn {
            0.0
        } else {
            audio_settings.volume_music.as_f32()
                * audio_settings.volume_master.as_f32()
        };

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
