use crate::metrics;
use bevy::audio::SpatialScale;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use unaudiospatial_core::events::SoundEvent;
use unaudiospatial_core::listener::SpatialListener;
use unmetrics_core::metrics::SendMetric;
use unsettings_core::audio::{AudioSettings, SoundOutput};
use unspatial_core::perspective;
use unspatial_core::position::Position;

pub fn spatial_audio_playback(
    mut sound_events: MessageReader<SoundEvent>,
    asset_server: Res<AssetServer>,
    qp: Query<&Position, With<SpatialListener>>,
    mut commands: Commands,
    audio_settings: Res<Persistent<AudioSettings>>,
    time: Res<Time>,
    mut last_error_log: Local<f32>,
) {
    let measure = metrics::SOUND_PLAYBACK.time_measure();
    let now = time.elapsed_secs();
    let mut can_log = now - *last_error_log > 1.0;
    let Ok(player_position) = qp.single() else {
        warn!("player not found!");
        measure.end_ms();
        return;
    };
    for sound_event in sound_events.read() {
        if !player_position.is_finite() && can_log {
            error!("Player position is not finite: {player_position:?}");
            *last_error_log = now;
            can_log = false;
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
            let mut spos_vec = perspective::to_screen_coord(position);
            spos_vec.z -= 10.0 / audio_settings.sound_output.to_ear_offset();
            sound.insert(Transform::from_translation(spos_vec));
        }
    }
    measure.end_ms();
}
