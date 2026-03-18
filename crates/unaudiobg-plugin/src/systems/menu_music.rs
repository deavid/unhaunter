use bevy::prelude::*;
use bevy_persistent::Persistent;
use unsettings_core::audio::AudioSettings;
use untypes_core::states::AppState;

use unaudiobg_core::smooth::smooth_volume_db;

/// Component marking an entity as playing menu music.
#[derive(Component, Debug, Default)]
pub(crate) struct MenuSound {
    despawn: bool,
}

/// Unified dB smoothing rate for both menu music and background tracks
const DB_PER_SECOND: f32 = 10.0;

/// Manages menu music spawning and lifecycle based on AppState.
/// Spawns menu music when not in game, triggers despawn when entering game.
pub(crate) fn manage_title_song(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut q_sound: Query<&mut MenuSound>,
    app_state: Res<State<AppState>>,
    audio_settings: Res<Persistent<AudioSettings>>,
    global_volume: Res<bevy::audio::GlobalVolume>,
) {
    let should_play_song = !matches!(app_state.get(), AppState::InGame);

    if let Ok(mut menusound) = q_sound.single_mut() {
        if !should_play_song && !menusound.despawn {
            menusound.despawn = true;
        } else if should_play_song && menusound.despawn {
            menusound.despawn = false;
        }
    } else if should_play_song {
        // Only spawn the song if the volume is greater than 0
        let desired_volume = audio_settings.volume_music.as_f32()
            * audio_settings.volume_master.as_f32()
            * global_volume.volume.to_linear();
        if desired_volume > 0.0 {
            commands
                .spawn(MenuSound::default())
                .insert(AudioPlayer::<AudioSource>(
                    asset_server.load("music/unhaunter_intro.ogg"),
                ))
                .insert(PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop,
                    volume: bevy::audio::Volume::Linear(desired_volume),
                    speed: 1.0,
                    paused: false,
                    spatial: false,
                    spatial_scale: None,
                    ..default()
                });
        }
    }
}

/// Manages menu music volume transitions using unified dB smoothing.
/// Fades out when MenuSound.despawn is true, fades in otherwise.
pub(crate) fn despawn_sound(
    mut commands: Commands,
    mut qs: Query<(Entity, &mut AudioSink, &MenuSound)>,
    audio_settings: Res<Persistent<AudioSettings>>,
    global_volume: Res<bevy::audio::GlobalVolume>,
    time: Res<Time>,
) {
    for (entity, mut sink, menusound) in &mut qs {
        let current_linear = sink.volume().to_linear();

        let target_linear = if menusound.despawn {
            0.0
        } else {
            audio_settings.volume_music.as_f32()
                * audio_settings.volume_master.as_f32()
                * global_volume.volume.to_linear()
        };

        let new_volume = smooth_volume_db(
            current_linear,
            target_linear,
            DB_PER_SECOND,
            time.delta_secs(),
        );

        sink.set_volume(bevy::audio::Volume::Linear(new_volume));

        if new_volume < 0.001 && menusound.despawn {
            commands.entity(entity).despawn();
            debug!("Menu song despawned");
        }
    }
}
