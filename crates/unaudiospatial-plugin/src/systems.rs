use crate::metrics;
use crate::plugin::UnSpatialPool;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use bevy_seedling::firewheel::dsp::distance_attenuation::DistanceAttenuation;
use bevy_seedling::nodes::itd::{ItdConfig, ItdNode};
use bevy_seedling::prelude::*;
use unaudiospatial_core::components::{
    AudioCategory, FlatAudio, SpatialAudioDelayedDespawn, SpatialAudioInstance,
};
use unaudiospatial_core::events::{LocalSoundEvent, SoundEvent};
use unaudiospatial_core::listener::SpatialListener;
use unmetrics_core::metrics::SendMetric;
use unreplicon_core::messages::ReplicatedSoundEvent;
use unreplicon_core::ownership::{LocallyOwned, Owner};
use untruck_core::components::in_truck::InTruck;
use unsettings_core::audio::{AudioSettings, SoundOutput};
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;

/// Minimum frame gap between two plays of the same sound to not be considered spam.
const AUDIO_SPAM_FRAME_THRESHOLD: u32 = 4;

pub(crate) struct SpatialParams {
    pub offset: bevy_seedling::firewheel::vector::Vec3,
    pub direction: Vec3,
    pub muffle_cutoff_hz: f32,
    pub panning_threshold: f32,
}

pub(crate) fn compute_spatial_params(
    player_position: &Position,
    player_dir: &Direction,
    sound_position: Option<Position>,
    is_mono: bool,
) -> SpatialParams {
    let aim = Vec2::new(player_dir.dx, player_dir.dy);
    let fwd = aim.normalize_or_zero();
    let fwd = if fwd == Vec2::ZERO { Vec2::Y } else { fwd };
    let right = Vec2::new(fwd.y, -fwd.x);

    let dist = sound_position
        .map(|pos| player_position.distance_zf(&pos, 12.5))
        .unwrap_or(0.0);

    let pos_val = sound_position.unwrap_or(*player_position);
    let delta = pos_val.delta(*player_position);

    let damp_lr = (dist / 2.0).clamp(0.0, 1.0);
    let damp_fb = (dist / 0.25).clamp(0.0, 1.0);

    let o_local_x = delta.dx * right.x + delta.dy * right.y;
    let o_local_y = delta.dx * fwd.x + delta.dy * fwd.y;

    let local_x = o_local_x * damp_lr;
    let local_y = o_local_y * damp_fb;
    let local_z = delta.dz * 12.5 * damp_lr + 0.5;

    let length_2 = (local_x * local_x + local_y * local_y + local_z * local_z).max(0.0000001);
    let length = length_2.sqrt();
    let far_enough = (length_2 / 4.0).tanh();
    let direction = Vec3::new(
        (local_x / length * far_enough).powi(3),
        local_y / length,
        local_z / length,
    )
    .normalize();

    let dist_ratio = (dist / 100.0).clamp(0.0, 1.0);
    let base_muffle_hz = (9.9034f32 - (5.9914f32 * dist_ratio)).exp();

    let inv_behind = if direction.y < 0.0 {
        let behind_factor = -direction.y; // 0.0 at sides, 1.0 directly behind
        let penalty_curve = behind_factor * behind_factor;
        penalty_curve / 800.0 // At 180 degrees, pulls constraint down precisely towards ~800Hz
    } else {
        0.0
    };

    let inv_floor = if delta.dz.abs() > 0.8 {
        let z_factor = ((delta.dz.abs() - 0.8) / 0.7).clamp(0.0, 1.0); // Full penalty by 1.5 tiles up/down
        z_factor / 400.0 // Floors muffle extremely heavily (towards ~400Hz max)
    } else {
        0.0
    };

    let muffle_cutoff_hz =
        (1.0 / ((1.0 / base_muffle_hz) + inv_behind + inv_floor)).clamp(20.0, 20_480.0);

    let is_spatial = sound_position.is_some() && !is_mono;

    let offset = if is_spatial {
        bevy_seedling::firewheel::vector::Vec3::new(local_x, local_y, local_z)
    } else {
        bevy_seedling::firewheel::vector::Vec3::new(0.0, 0.0, 0.0)
    };

    SpatialParams {
        offset,
        direction,
        muffle_cutoff_hz,
        panning_threshold: 0.3 * far_enough,
    }
}

fn play_sound_event(
    sound_event: &SoundEvent,
    player_position: &Position,
    player_dir: &Direction,
    audio_settings: &Persistent<AudioSettings>,
    asset_server: &AssetServer,
    commands: &mut Commands,
    q_instances: &Query<(Entity, &SpatialAudioInstance), Without<SpatialAudioDelayedDespawn>>,
    cur_frame: u32,
    now: f32,
) {
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
                commands
                    .entity(entity)
                    .insert(SpatialAudioDelayedDespawn::new(0.3));
            }
        }
    }

    if is_spam {
        return;
    }

    let dist = sound_event
        .position
        .map(|pos| player_position.distance_zf(&pos, 12.5))
        .unwrap_or(0.0);

    let is_mono = audio_settings.sound_output == SoundOutput::Mono;
    let params = compute_spatial_params(player_position, player_dir, sound_event.position, is_mono);

    let base_vol = sound_event.volume
        * audio_settings.volume_effects.as_f32()
        * audio_settings.volume_master.as_f32();

    let mut dry_vol = base_vol.clamp(0.0, 1.0);

    let rev_max_ratio = 2.0;
    let rev_ramp = (dist / 20.0).clamp(0.0, 1.0);
    let mut rev_vol = (base_vol * rev_max_ratio * rev_ramp).clamp(0.0, 1.0);

    if is_mono {
        let mono_div = 1.0 + dist * 0.4;
        dry_vol /= mono_div;
        rev_vol /= mono_div;
    }

    let direction = params.direction;
    let offset = params.offset;
    let muffle_hz = params.muffle_cutoff_hz;
    let panning_threshold = params.panning_threshold;

    trace!("ITD direction: {direction:?}");

    commands.spawn((
        SpatialAudioInstance {
            sound_file: sound_event.sound_file.clone(),
            is_reverb: false,
            initial_volume: dry_vol,
            spawn_time: now,
            spawn_frame: cur_frame,
            position: sound_event.position,
        },
        SamplePlayer::new(asset_server.load(sound_event.sound_file.clone())),
        sample_effects![
            VolumeNode {
                volume: Volume::Linear(dry_vol),
                smooth_seconds: 0.0005,
                ..default()
            },
            SpatialBasicNode {
                offset,
                muffle_cutoff_hz: muffle_hz,
                smooth_seconds: 0.0005,
                downmix: false,
                panning_threshold,
                ..default()
            },
            (
                ItdNode { direction },
                ItdConfig {
                    inter_ear_distance: 0.11,
                    ..default()
                }
            )
        ],
        UnSpatialPool,
    ));

    if rev_vol > 0.001 {
        commands.spawn((
            SpatialAudioInstance {
                sound_file: reverb_file.clone(),
                is_reverb: true,
                initial_volume: rev_vol,
                spawn_time: now,
                spawn_frame: cur_frame,
                position: sound_event.position,
            },
            SamplePlayer::new(asset_server.load(reverb_file)),
            sample_effects![
                VolumeNode {
                    volume: Volume::Linear(rev_vol),
                    smooth_seconds: 0.0005,
                    ..default()
                },
                SpatialBasicNode {
                    offset,
                    muffle_cutoff_hz: muffle_hz,
                    distance_attenuation: DistanceAttenuation {
                        distance_gain_factor: 0.25,
                        ..default()
                    },
                    smooth_seconds: 0.0005,
                    downmix: false,
                    panning_threshold,
                    ..default()
                },
                (
                    ItdNode { direction },
                    ItdConfig {
                        inter_ear_distance: 0.11,
                        ..default()
                    }
                )
            ],
            UnSpatialPool,
        ));
    }
}

pub fn local_spatial_audio_playback(
    mut sound_events: MessageReader<LocalSoundEvent>,
    asset_server: Res<AssetServer>,
    qp: Query<(&Position, &Direction), With<SpatialListener>>,
    mut commands: Commands,
    audio_settings: Res<Persistent<AudioSettings>>,
    time: Res<Time>,
    mut last_error_log: Local<f32>,
    q_instances: Query<(Entity, &SpatialAudioInstance), Without<SpatialAudioDelayedDespawn>>,
) {
    let measure = metrics::SOUND_PLAYBACK.time_measure();
    let now = time.elapsed_secs();
    let cur_frame = (now * 60.0) as u32;
    let mut can_log = now - *last_error_log > 1.0;
    let Ok((player_position, player_dir)) = qp.single() else {
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

        let replicated_view = SoundEvent::from(sound_event);
        play_sound_event(
            &replicated_view,
            player_position,
            player_dir,
            &audio_settings,
            &asset_server,
            &mut commands,
            &q_instances,
            cur_frame,
            now,
        );
    }
    measure.end_ms();
}

pub fn spatial_audio_playback(
    mut sound_events: MessageReader<SoundEvent>,
    asset_server: Res<AssetServer>,
    qp: Query<(&Position, &Direction), With<SpatialListener>>,
    mut commands: Commands,
    audio_settings: Res<Persistent<AudioSettings>>,
    time: Res<Time>,
    mut last_error_log: Local<f32>,
    q_instances: Query<(Entity, &SpatialAudioInstance), Without<SpatialAudioDelayedDespawn>>,
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

    for sound_event in sound_events.read() {
        if !player_position.is_finite() && can_log {
            error!("Player position is not finite: {player_position:?}");
            *last_error_log = now;
            can_log = false;
        }

        play_sound_event(
            sound_event,
            player_position,
            player_dir,
            &audio_settings,
            &asset_server,
            &mut commands,
            &q_instances,
            cur_frame,
            now,
        );
    }
    measure.end_ms();
}

pub fn process_audio_delayed_despawns(
    mut commands: Commands,
    time: Res<Time>,
    mut q_fadeouts: Query<(Entity, &mut SpatialAudioDelayedDespawn)>,
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
    if count > 30 && now - *last_warn > 1.0 {
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

pub fn replicated_spatial_audio_playback(
    mut sound_events: MessageReader<ReplicatedSoundEvent>,
    asset_server: Res<AssetServer>,
    qp: Query<(&Position, &Direction, Has<InTruck>), With<SpatialListener>>,
    mut commands: Commands,
    audio_settings: Res<Persistent<AudioSettings>>,
    time: Res<Time>,
    mut last_error_log: Local<f32>,
    q_instances: Query<(Entity, &SpatialAudioInstance), Without<SpatialAudioDelayedDespawn>>,
    q_local_player: Query<&Owner, With<LocallyOwned>>,
) {
    let measure = metrics::SOUND_PLAYBACK.time_measure();
    let now = time.elapsed_secs();
    let cur_frame = (now * 60.0) as u32;
    let mut can_log = now - *last_error_log > 1.0;

    let Ok((player_position, player_dir, listener_in_truck)) = qp.single() else {
        if can_log {
            warn!("player not found!");
            *last_error_log = now;
        }
        measure.end_ms();
        return;
    };

    let local_owner_id = q_local_player.single().ok().map(|o| o.0);

    for replicated_ev in sound_events.read() {
        if !player_position.is_finite() && can_log {
            error!("Player position is not finite: {player_position:?}");
            *last_error_log = now;
            can_log = false;
        }

        let mut volume = replicated_ev.volume;
        let sound_pos = replicated_ev.position.map(|p| Position {
            x: p[0],
            y: p[1],
            z: p[2],
            ..Default::default()
        });

        // Triggerer Identification: Penalize those who are not the one who triggered it.
        let is_triggerer = local_owner_id == Some(replicated_ev.triggerer);

        if !is_triggerer {
            // Penalization for those that are not the ones to trigger it.
            // Penalization in the form of severe muffling and lower volume.
            volume *= 0.4;
        }

        let mut sound_event = SoundEvent {
            sound_file: replicated_ev.sound_file.clone(),
            volume,
            position: sound_pos,
        };

        // If the sound happens IN the van, for people OUT of the van should get even more penalty
        if replicated_ev.is_inside_truck && !listener_in_truck {
            sound_event.volume *= 0.2;
        }

        let is_mono = audio_settings.sound_output == SoundOutput::Mono;
        let mut params =
            compute_spatial_params(player_position, player_dir, sound_event.position, is_mono);

        // Apply aggressive muffling if not triggerer or if sound is inside truck and listener is outside.
        if !is_triggerer {
            params.muffle_cutoff_hz = params.muffle_cutoff_hz.min(1200.0);
        }
        if replicated_ev.is_inside_truck && !listener_in_truck {
            params.muffle_cutoff_hz = params.muffle_cutoff_hz.min(400.0);
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
                    commands
                        .entity(entity)
                        .insert(SpatialAudioDelayedDespawn::new(0.3));
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

        let base_vol = sound_event.volume
            * audio_settings.volume_effects.as_f32()
            * audio_settings.volume_master.as_f32();

        let mut dry_vol = base_vol.clamp(0.0, 1.0);

        let rev_max_ratio = 2.0;
        let rev_ramp = (dist / 20.0).clamp(0.0, 1.0);
        let mut rev_vol = (base_vol * rev_max_ratio * rev_ramp).clamp(0.0, 1.0);

        if is_mono {
            let mono_div = 1.0 + dist * 0.4;
            dry_vol /= mono_div;
            rev_vol /= mono_div;
        }

        let direction = params.direction;
        let offset = params.offset;
        let muffle_hz = params.muffle_cutoff_hz;
        let panning_threshold = params.panning_threshold;

        commands.spawn((
            SpatialAudioInstance {
                sound_file: sound_event.sound_file.clone(),
                is_reverb: false,
                initial_volume: dry_vol,
                spawn_time: now,
                spawn_frame: cur_frame,
                position: sound_event.position,
            },
            SamplePlayer::new(asset_server.load(sound_event.sound_file.clone())),
            sample_effects![
                VolumeNode {
                    volume: Volume::Linear(dry_vol),
                    smooth_seconds: 0.0005,
                    ..default()
                },
                SpatialBasicNode {
                    offset,
                    muffle_cutoff_hz: muffle_hz,
                    smooth_seconds: 0.0005,
                    downmix: false,
                    panning_threshold,
                    ..default()
                },
                (
                    ItdNode { direction },
                    ItdConfig {
                        inter_ear_distance: 0.11,
                        ..default()
                    }
                )
            ],
            UnSpatialPool,
        ));

        if rev_vol > 0.001 {
            commands.spawn((
                SpatialAudioInstance {
                    sound_file: reverb_file.clone(),
                    is_reverb: true,
                    initial_volume: rev_vol,
                    spawn_time: now,
                    spawn_frame: cur_frame,
                    position: sound_event.position,
                },
                SamplePlayer::new(asset_server.load(reverb_file)),
                sample_effects![
                    VolumeNode {
                        volume: Volume::Linear(rev_vol),
                        smooth_seconds: 0.0005,
                        ..default()
                    },
                    SpatialBasicNode {
                        offset,
                        muffle_cutoff_hz: muffle_hz,
                        distance_attenuation: DistanceAttenuation {
                            distance_gain_factor: 0.25,
                            ..default()
                        },
                        smooth_seconds: 0.0005,
                        downmix: false,
                        panning_threshold,
                        ..default()
                    },
                    (
                        ItdNode { direction },
                        ItdConfig {
                            inter_ear_distance: 0.11,
                            ..default()
                        }
                    )
                ],
                UnSpatialPool,
            ));
        }
    }
    measure.end_ms();
}

pub fn update_spatial_audio(
    qp: Query<(&Position, &Direction), With<SpatialListener>>,
    audio_settings: Res<Persistent<AudioSettings>>,
    q_audio: Query<(&SpatialAudioInstance, &SampleEffects)>,
    mut q_basic: Query<&mut SpatialBasicNode>,
    mut q_itd: Query<&mut ItdNode>,
) {
    let Ok((player_position, player_dir)) = qp.single() else {
        return;
    };

    let is_mono = audio_settings.sound_output == SoundOutput::Mono;

    for (instance, effects) in q_audio.iter() {
        let params =
            compute_spatial_params(player_position, player_dir, instance.position, is_mono);

        if let Ok(mut basic_node) = q_basic.get_effect_mut(effects) {
            basic_node.offset = params.offset;
            basic_node.muffle_cutoff_hz = params.muffle_cutoff_hz;
            basic_node.panning_threshold = params.panning_threshold;
            basic_node.smooth_seconds = 0.02; // 20ms during updates
        }

        if let Ok(mut itd_node) = q_itd.get_effect_mut(effects) {
            itd_node.direction = params.direction;
        }
    }
}
