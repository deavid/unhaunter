use bevy::prelude::*;
use unaudiobg_core::components::{GameSound, SoundType};

/// Settings for spawning background audio tracks
struct PlaybackSettingsTemplate;

impl PlaybackSettingsTemplate {
    /// Creates standard PlaybackSettings for background audio tracks
    fn for_background_track() -> PlaybackSettings {
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::Linear(0.00001),
            speed: 1.0,
            paused: false,
            spatial: false,
            spatial_scale: None,
            ..default()
        }
    }
}

/// Spawns all four background audio track entities at startup, initially silent.
/// These entities are never despawned; their volumes are controlled by the audio system.
pub(crate) fn spawn_background_tracks(mut commands: Commands, asset_server: Res<AssetServer>) {
    // BackgroundHouse
    commands
        .spawn(AudioPlayer::new(
            asset_server.load("sounds/background-noise-house-1.ogg"),
        ))
        .insert(PlaybackSettingsTemplate::for_background_track())
        .insert(GameSound {
            class: SoundType::BackgroundHouse,
        });

    // BackgroundStreet
    commands
        .spawn(AudioPlayer::new(
            asset_server.load("sounds/ambient-clean.ogg"),
        ))
        .insert(PlaybackSettingsTemplate::for_background_track())
        .insert(GameSound {
            class: SoundType::BackgroundStreet,
        });

    // HeartBeat
    commands
        .spawn(AudioPlayer::new(
            asset_server.load("sounds/heartbeat-1.ogg"),
        ))
        .insert(PlaybackSettingsTemplate::for_background_track())
        .insert(GameSound {
            class: SoundType::HeartBeat,
        });

    // Insane
    commands
        .spawn(AudioPlayer::new(asset_server.load("sounds/insane-1.ogg")))
        .insert(PlaybackSettingsTemplate::for_background_track())
        .insert(GameSound {
            class: SoundType::Insane,
        });
}

/// Silences all background audio tracks when exiting InGame state.
/// Sets volumes to zero without despawning entities.
pub(crate) fn silence_background_tracks(
    mut game_sound_query: Query<&mut AudioSink, With<GameSound>>,
) {
    for mut audio_sink in &mut game_sound_query {
        audio_sink.set_volume(bevy::audio::Volume::Linear(0.0));
    }
}
