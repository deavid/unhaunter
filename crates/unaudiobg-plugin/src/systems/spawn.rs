use bevy::prelude::*;
use bevy_seedling::prelude::*;
use unaudiobg_core::components::{GameSound, SoundType};

/// Spawns all four background audio track entities at startup, initially silent.
/// These entities are never despawned; their volumes are controlled by the audio system.
pub(crate) fn spawn_background_tracks(mut commands: Commands, asset_server: Res<AssetServer>) {
    let initial_volume = Volume::Linear(0.00001);

    // BackgroundHouse
    commands.spawn((
        SamplePlayer::new(asset_server.load("sounds/background-noise-house-1.ogg")).looping(),
        sample_effects![VolumeNode {
            volume: initial_volume,
            ..default()
        }],
        GameSound {
            class: SoundType::BackgroundHouse,
        },
        SoundEffectsBus,
    ));

    // BackgroundStreet
    commands.spawn((
        SamplePlayer::new(asset_server.load("sounds/ambient-clean.ogg")).looping(),
        sample_effects![VolumeNode {
            volume: initial_volume,
            ..default()
        }],
        GameSound {
            class: SoundType::BackgroundStreet,
        },
        SoundEffectsBus,
    ));

    // HeartBeat
    commands.spawn((
        SamplePlayer::new(asset_server.load("sounds/heartbeat-1.ogg")).looping(),
        sample_effects![VolumeNode {
            volume: initial_volume,
            ..default()
        }],
        GameSound {
            class: SoundType::HeartBeat,
        },
        SoundEffectsBus,
    ));

    // Insane
    commands.spawn((
        SamplePlayer::new(asset_server.load("sounds/insane-1.ogg")).looping(),
        sample_effects![VolumeNode {
            volume: initial_volume,
            ..default()
        }],
        GameSound {
            class: SoundType::Insane,
        },
        SoundEffectsBus,
    ));
}

/// Silences all background audio tracks when exiting InGame state.
/// Sets volumes to zero without despawning entities.
pub(crate) fn silence_background_tracks(mut game_sound_query: Query<&mut VolumeNode, With<GameSound>>) {
    for mut volume_node in &mut game_sound_query {
        volume_node.volume = Volume::Linear(0.0);
    }
}
