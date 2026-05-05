use crate::metrics;
use bevy::audio::SpatialScale;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use unaudiospatial_core::components::{SpatialAudioFadeOut, SpatialAudioInstance};
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
    q_instances: Query<(Entity, &SpatialAudioInstance), Without<SpatialAudioFadeOut>>,
) {
    let measure = metrics::SOUND_PLAYBACK.time_measure();
    let now = time.elapsed_secs();
    let cur_frame = (now * 60.0) as u32; // Fallback to time-based pseudo-frames
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

        // Deduplication & Spam Detection
        let mut is_spam = false;
        let reverb_file = sound_event.sound_file.replace("sounds/", "reverbs/");

        for (entity, instance) in q_instances.iter() {
            if instance.sound_file == sound_event.sound_file || instance.sound_file == reverb_file {
                let frame_diff = cur_frame.saturating_sub(instance.spawn_frame);
                if !instance.is_reverb && frame_diff < 5 {
                    warn!(
                        "AUDIO SPAM: Sound '{}' triggered too rapidly ({} frames). Skipping.",
                        sound_event.sound_file, frame_diff
                    );
                    is_spam = true;
                    break;
                } else {
                    // Not spam, but older instance: trigger fade out
                    commands
                        .entity(entity)
                        .insert(SpatialAudioFadeOut::new(0.3));
                }
            }
        }

        if is_spam {
            continue;
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
        let rev_proximity_fade = ((dist - 2.0) / 10.0).clamp(0.0, 1.0);
        let mut rev_vol =
            (base_vol * rev_fade * rev_compensation * rev_proximity_fade).clamp(0.0, 1.0);

        if audio_settings.sound_output == SoundOutput::Mono {
            let mono_div = 1.0 + dist * 0.4;
            dry_vol /= mono_div;
            rev_vol /= mono_div;
        }

        // --- DRY LAYER ---
        let dry_id = commands
            .spawn((
                SpatialAudioInstance {
                    sound_file: sound_event.sound_file.clone(),
                    is_reverb: false,
                    initial_volume: dry_vol,
                    spawn_time: now,
                    spawn_frame: cur_frame,
                },
                AudioPlayer::<AudioSource>(asset_server.load(sound_event.sound_file.clone())),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::Linear(dry_vol),
                    speed: 1.0,
                    paused: false,
                    spatial: sound_event.position.is_some()
                        && audio_settings.sound_output != SoundOutput::Mono,
                    spatial_scale: Some(SpatialScale::new(0.005)),
                    ..default()
                },
            ))
            .id();

        // --- REVERB LAYER ---
        let rev_id = commands
            .spawn((
                SpatialAudioInstance {
                    sound_file: reverb_file.clone(),
                    is_reverb: true,
                    initial_volume: rev_vol,
                    spawn_time: now,
                    spawn_frame: cur_frame,
                },
                AudioPlayer::<AudioSource>(asset_server.load(reverb_file)),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::Linear(rev_vol),
                    speed: 1.0,
                    paused: false,
                    spatial: sound_event.position.is_some()
                        && audio_settings.sound_output != SoundOutput::Mono,
                    // Smaller scale makes the reverb feel "wider" and less like a point source
                    spatial_scale: Some(SpatialScale::new(0.001)),
                    ..default()
                },
            ))
            .id();

        if let Some(position) = sound_event.position {
            let mut spos_vec = perspective::to_screen_coord(position);
            spos_vec.z -= 10.0 / audio_settings.sound_output.to_ear_offset();
            let transform = Transform::from_translation(spos_vec);
            commands.entity(dry_id).insert(transform);
            commands.entity(rev_id).insert(transform);
        }
    }
    measure.end_ms();
}

pub fn process_audio_fadeouts(
    mut commands: Commands,
    time: Res<Time>,
    mut q_fadeouts: Query<(
        Entity,
        &mut SpatialAudioFadeOut,
        &SpatialAudioInstance,
        Option<&mut AudioSink>,
    )>,
) {
    for (entity, mut fadeout, instance, sink) in q_fadeouts.iter_mut() {
        fadeout.timer.tick(time.delta());
        if fadeout.timer.just_finished() {
            commands.entity(entity).despawn();
        } else if let Some(mut sink) = sink {
            let volume = instance.initial_volume * (1.0 - fadeout.timer.fraction());
            sink.set_volume(Volume::Linear(volume));
        }
    }
}

pub fn monitor_audio_pileup(
    q_audio_players: Query<&AudioPlayer<AudioSource>>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut last_warn: Local<f32>,
) {
    let count = q_audio_players.iter().count();
    let now = time.elapsed_secs();
    if count > 20 && now - *last_warn > 1.0 {
        let mut counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::default();
        for player in q_audio_players.iter() {
            let path = asset_server
                .get_path(&player.0)
                .map(|p| p.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            *counts.entry(path).or_insert(0) += 1;
        }
        let mut details: Vec<_> = counts.into_iter().filter(|(_, c)| *c > 1).collect();
        details.sort_by(|a, b| b.1.cmp(&a.1));

        warn!(
            "AUDIO PILEUP: {} active AudioPlayer entities detected! Details: {:?}",
            count, details
        );
        *last_warn = now;
    }
}
