use crate::metrics;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use bevy_seedling::prelude::*;
use unaudiospatial_core::events::SoundEvent;
use unmetrics_core::metrics::SendMetric;
use unsettings_core::audio::{AudioSettings, SoundOutput};
use unspatial_core::perspective;
use unspatial_core::position::Position;

pub fn spatial_audio_playback(
    mut sound_events: MessageReader<SoundEvent>,
    asset_server: Res<AssetServer>,
    qp: Query<&Position, With<SpatialListener2D>>,
    mut commands: Commands,
    audio_settings: Res<Persistent<AudioSettings>>,
    time: Res<Time>,
    mut last_error_log: Local<f32>,
) {
    let measure = metrics::SOUND_PLAYBACK.time_measure();
    let now = time.elapsed_secs();
    let mut can_log = now - *last_error_log > 1.0;
    let Ok(player_position) = qp.single() else {
        if can_log {
            warn!("player not found!");
            *last_error_log = now;
        }
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

        let base_vol = sound_event.volume
            * audio_settings.volume_effects.as_f32()
            * audio_settings.volume_master.as_f32();

        // Dry Volume: Pure natural decay (no compensation)
        let dry_fade = (1.0 - (dist / 15.0).powi(2)).max(0.0);
        let mut dry_vol = (base_vol * dry_fade).clamp(0.0, 1.0);

        // Reverb Volume: Stronger compensation to counteract Bevy and 'bloom' over distance
        let rev_fade = 0.6 / (1.0 + dist * 0.05);
        let rev_compensation = 1.0 + dist * 0.4;
        let rev_compensation = rev_compensation.clamp(1.0, 4.0);
        let rev_proximity_fade = ((dist - 2.0) / 10.0).clamp(0.0, 1.0);
        let mut rev_vol =
            (base_vol * rev_fade * rev_compensation * rev_proximity_fade).clamp(0.0, 1.0);

        if audio_settings.sound_output == SoundOutput::Mono {
            let mono_div = 1.0 + dist * 0.4;
            dry_vol /= mono_div;
            rev_vol /= mono_div;
        }

        let is_spatial = sound_event.position.is_some()
            && audio_settings.sound_output != SoundOutput::Mono;

        let mut transform = None;
        if let Some(position) = sound_event.position {
            let mut spos_vec = perspective::to_screen_coord(position);
            spos_vec.z -= 10.0 / audio_settings.sound_output.to_ear_offset();
            transform = Some(Transform::from_translation(spos_vec));
        }

        // --- DRY LAYER ---
        let mut dry_cmd = commands.spawn((
            SamplePlayer::new(asset_server.load(sound_event.sound_file.clone())),
            sample_effects![
                VolumeNode {
                    volume: Volume::Linear(dry_vol),
                    ..default()
                },
                SpatialBasicNode::default()
            ],
            SpatialPool,
        ));

        if is_spatial {
            if let Some(t) = transform {
                dry_cmd.insert(t);
            }
        }

        // --- REVERB LAYER ---
        let reverb_file = sound_event.sound_file.replace("sounds/", "reverbs/");
        let mut rev_cmd = commands.spawn((
            SamplePlayer::new(asset_server.load(reverb_file)),
            sample_effects![
                VolumeNode {
                    volume: Volume::Linear(rev_vol),
                    ..default()
                },
                SpatialBasicNode::default()
            ],
            SpatialPool,
        ));

        if is_spatial {
            if let Some(t) = transform {
                rev_cmd.insert(t);
            }
        }
    }
    measure.end_ms();
}
