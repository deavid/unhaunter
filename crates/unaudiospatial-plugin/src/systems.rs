use crate::metrics;
use crate::plugin::UnSpatialPool;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use bevy_seedling::firewheel::dsp::distance_attenuation::DistanceAttenuation;
use bevy_seedling::nodes::itd::{ItdConfig, ItdNode};
use bevy_seedling::prelude::*;
use unaudiospatial_core::components::{
    AudioCategory, FlatAudio, SpatialAudioFadeOut, SpatialAudioInstance,
};
use unaudiospatial_core::events::SoundEvent;
use unaudiospatial_core::listener::SpatialListener;
use unmetrics_core::metrics::SendMetric;
use unsettings_core::audio::{AudioSettings, SoundOutput};
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;

/// Minimum frame gap between two plays of the same sound to not be considered spam.
const AUDIO_SPAM_FRAME_THRESHOLD: u32 = 5;

pub fn spatial_audio_playback(
    mut sound_events: MessageReader<SoundEvent>,
    asset_server: Res<AssetServer>,
    qp: Query<(&Position, &Direction), With<SpatialListener>>,
    mut commands: Commands,
    audio_settings: Res<Persistent<AudioSettings>>,
    time: Res<Time>,
    mut last_error_log: Local<f32>,
    q_instances: Query<(Entity, &SpatialAudioInstance), Without<SpatialAudioFadeOut>>,
) {
    let measure = metrics::SOUND_PLAYBACK.time_measure();
    let now = time.elapsed_secs();
    let cur_frame = (now * 60.0) as u32; // Time-based pseudo-frame counter
    let mut can_log = now - *last_error_log > 1.0;
    let Ok((player_position, player_dir)) = qp.single() else {
        if can_log {
            warn!("player not found!");
            *last_error_log = now;
        }
        measure.end_ms();
        return;
    };

    let aim = Vec2::new(player_dir.dx, player_dir.dy);
    let fwd = aim.normalize_or_zero();
    let fwd = if fwd == Vec2::ZERO { Vec2::Y } else { fwd };
    let right = Vec2::new(fwd.y, -fwd.x);

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
                if !instance.is_reverb && frame_diff < AUDIO_SPAM_FRAME_THRESHOLD {
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
            .map(|pos| player_position.distance_zf(&pos, 12.5))
            .unwrap_or(0.0);

        let pos_val = sound_event.position.unwrap_or(*player_position);
        let delta = pos_val.delta(*player_position);

        // Apply spatial dampening for close sounds so they don't pan extremely.
        // Front-back needs ~0.25 units to be highly noticeable, left-right ~2.0 units.
        let damp_lr = (dist / 2.0).clamp(0.0, 1.0);
        let damp_fb = (dist / 0.25).clamp(0.0, 1.0);

        let o_local_x = delta.dx * right.x + delta.dy * right.y;
        let o_local_y = delta.dx * fwd.x + delta.dy * fwd.y;

        let local_x = o_local_x * damp_lr;
        let local_y = o_local_y * damp_fb;
        let local_z = delta.dz * 12.5 * damp_lr;

        // normalize direction for ITD
        let length = (local_x * local_x + local_y * local_y + local_z * local_z)
            .sqrt()
            .max(0.0001);
        let direction = Vec3::new(local_x / length, local_y / length, local_z / length);

        let base_vol = sound_event.volume
            * audio_settings.volume_effects.as_f32()
            * audio_settings.volume_master.as_f32();

        // Dry Volume: We let Firewheel's spatializer apply distance attenuation natively via offset.
        let mut dry_vol = base_vol.clamp(0.0, 1.0);

        // Reverb Volume: Ramps up to a maximum multiplier over 20 tiles (bloom),
        // and relies on Firewheel's spatializer for the actual distance falloff.
        let rev_max_ratio = 2.0; // Boosted so it can be heard
        let rev_ramp = (dist / 20.0).clamp(0.0, 1.0);
        let mut rev_vol = (base_vol * rev_max_ratio * rev_ramp).clamp(0.0, 1.0);

        // --- MUFFLING CALCULATION (Parallel resistance formula) ---
        // We use 1/M_total = 1/M_dist + 1/M_behind + 1/M_floor.
        // This ensures the heaviest muffle always dominates, but they combined naturally.

        // 1. Distance Muffle Component
        let dist_ratio = (dist / 200.0).clamp(0.0, 1.0);
        let base_rev_muffle_hz = (9.9034f32 - (5.9914f32 * dist_ratio)).exp();

        let dry_dist_ratio = (dist / 100.0).clamp(0.0, 1.0); // Dry muffles twice as fast
        let base_dry_muffle_hz = (9.9034f32 - (5.9914f32 * dry_dist_ratio)).exp();

        // 2. Behind Player Component (Only applies to Dry signal, Reverb is ambient)
        let inv_behind = if direction.y < 0.0 {
            let behind_factor = -direction.y; // 0.0 at sides, 1.0 directly behind
            let penalty_curve = behind_factor * behind_factor;
            penalty_curve / 800.0 // At 180 degrees, pulls constraint down precisely towards ~800Hz
        } else {
            0.0
        };

        // 3. Different Floor Component (Applies to both Dry and Reverb)
        let inv_floor = if delta.dz.abs() > 0.8 {
            let z_factor = ((delta.dz.abs() - 0.8) / 0.7).clamp(0.0, 1.0); // Full penalty by 1.5 tiles up/down
            z_factor / 400.0 // Floors muffle extremely heavily (towards ~400Hz max)
        } else {
            0.0
        };

        // Complete combinations!
        let rev_muffle_hz = (1.0 / ((1.0 / base_rev_muffle_hz) + inv_floor)).clamp(20.0, 20_480.0);
        let dry_muffle_hz =
            (1.0 / ((1.0 / base_dry_muffle_hz) + inv_behind + inv_floor)).clamp(20.0, 20_480.0);

        if audio_settings.sound_output == SoundOutput::Mono {
            let mono_div = 1.0 + dist * 0.4;
            dry_vol /= mono_div;
            rev_vol /= mono_div;
        }

        let is_spatial =
            sound_event.position.is_some() && audio_settings.sound_output != SoundOutput::Mono;

        let offset = if is_spatial {
            bevy_seedling::firewheel::vector::Vec3::new(local_x, local_y, local_z)
        } else {
            bevy_seedling::firewheel::vector::Vec3::new(0.0, 0.0, 0.0)
        };

        // --- DRY LAYER ---
        commands.spawn((
            SpatialAudioInstance {
                sound_file: sound_event.sound_file.clone(),
                is_reverb: false,
                initial_volume: dry_vol,
                spawn_time: now,
                spawn_frame: cur_frame,
            },
            SamplePlayer::new(asset_server.load(sound_event.sound_file.clone())),
            sample_effects![
                VolumeNode {
                    volume: Volume::Linear(dry_vol),
                    ..default()
                },
                SpatialBasicNode {
                    offset,
                    muffle_cutoff_hz: dry_muffle_hz,
                    smooth_seconds: 0.0005,
                    downmix: false,
                    panning_threshold: 0.3,
                    ..default()
                },
                (
                    ItdNode { direction },
                    ItdConfig {
                        inter_ear_distance: 0.1715,
                        ..default()
                    }
                )
            ],
            UnSpatialPool,
        ));

        // --- REVERB LAYER ---

        commands.spawn((
            SpatialAudioInstance {
                sound_file: reverb_file.clone(),
                is_reverb: true,
                initial_volume: rev_vol,
                spawn_time: now,
                spawn_frame: cur_frame,
            },
            SamplePlayer::new(asset_server.load(reverb_file)),
            sample_effects![
                VolumeNode {
                    volume: Volume::Linear(rev_vol),
                    ..default()
                },
                SpatialBasicNode {
                    offset,
                    muffle_cutoff_hz: rev_muffle_hz,
                    distance_attenuation: DistanceAttenuation {
                        distance_gain_factor: 0.25,
                        ..default()
                    },
                    smooth_seconds: 0.0005,
                    downmix: false,
                    panning_threshold: 0.3,
                    ..default()
                },
                (
                    ItdNode { direction },
                    ItdConfig {
                        inter_ear_distance: 0.1715,
                        ..default()
                    }
                )
            ],
            UnSpatialPool,
        ));
    }
    measure.end_ms();
}

pub fn process_audio_fadeouts(
    mut commands: Commands,
    time: Res<Time>,
    mut q_fadeouts: Query<(Entity, &mut SpatialAudioFadeOut)>,
) {
    for (entity, mut fadeout) in q_fadeouts.iter_mut() {
        fadeout.timer.tick(time.delta());
        if fadeout.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn monitor_audio_pileup(
    q_audio_players: Query<&SamplePlayer>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut last_warn: Local<f32>,
) {
    let count = q_audio_players.iter().count();
    let now = time.elapsed_secs();
    if count > 20 && now - *last_warn > 1.0 {
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for player in q_audio_players.iter() {
            let path = asset_server
                .get_path(&player.sample)
                .map(|p| p.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            *counts.entry(path).or_insert(0) += 1;
        }
        let mut details: Vec<_> = counts.into_iter().filter(|(_, c)| *c > 1).collect();
        details.sort_by(|a, b| b.1.cmp(&a.1));

        warn!(
            "AUDIO PILEUP: {} active SamplePlayer entities detected! Details: {:?}",
            count, details
        );
        *last_warn = now;
    }
}

pub(crate) fn attach_flat_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<Persistent<AudioSettings>>,
    q_flat: Query<(Entity, &FlatAudio), Added<FlatAudio>>,
) {
    for (entity, flat) in q_flat.iter() {
        let base_vol = audio_settings.volume_master.as_f32() * flat.volume_multiplier;
        let final_vol = match flat.category {
            AudioCategory::Effects => base_vol * audio_settings.volume_effects.as_f32(),
            AudioCategory::VoiceChat => base_vol * audio_settings.volume_voice_chat.as_f32(),
            AudioCategory::Master => base_vol,
        };

        commands.entity(entity).insert((
            SamplePlayer::new(asset_server.load(&flat.sound_file)),
            sample_effects![VolumeNode {
                volume: Volume::Linear(final_vol),
                ..default()
            }],
            DefaultPool,
        ));
    }
}
