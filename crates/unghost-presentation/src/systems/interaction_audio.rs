use bevy::prelude::*;
use unaudiospatial_core::emitter::AudioEmitter;
use uncommon_states_core::UIContextState;
use unghost_core::components::logic::interaction_sound::GhostInteractionSoundCue;
use unghost_core::components::logic::vocalization::GhostVocalization;
use unghost_core::events::GhostInteractionType;

/// Plays ghost interaction sounds locally when `GhostInteractionSoundCue` changes
/// on an entity (via Replicon replication from the authority).
fn play_interaction_sounds(
    q: Query<&GhostInteractionSoundCue, Changed<GhostInteractionSoundCue>>,
    mut audio: AudioEmitter,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs_f64();
    for cue in q.iter() {
        // Skip the default sentinel state (triggered_at == 0.0)
        if cue.triggered_at == 0.0 {
            continue;
        }
        // Skip stale cues to avoid replaying old sounds on late-join
        if current_time - cue.triggered_at > 5.0 {
            continue;
        }
        let (sound_file, volume) = match cue.kind {
            GhostInteractionType::DoorSlam => ("sounds/door-close.ogg", 1.5_f32),
            GhostInteractionType::DoorCreak => ("sounds/door_creak_slow.ogg", 0.7),
            GhostInteractionType::Throw => ("sounds/object_throw_generic.ogg", 0.8),
            GhostInteractionType::Nudge => ("sounds/object_nudge_1.ogg", 0.6),
            GhostInteractionType::HauntedMove => ("sounds/object_drag_wood.ogg", 0.9),
            GhostInteractionType::Lock => ("sounds/door_lock_heavy.ogg", 0.9),
            GhostInteractionType::TripBreaker => ("sounds/switch-on-2.ogg", 1.0),
            GhostInteractionType::Toggle => continue, // Toggle has no dedicated sound
        };
        audio.play_audio(sound_file.to_string(), volume, &cue.position);
    }
}

/// Plays ghost vocalization sounds locally when `GhostVocalization` changes
/// (via Replicon replication from the authority).
fn play_vocalization_sounds(
    q: Query<&GhostVocalization, Changed<GhostVocalization>>,
    mut audio: AudioEmitter,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs_f64();
    for voc in q.iter() {
        // Skip the default empty state
        if voc.sound_file.is_empty() {
            continue;
        }
        // Skip stale vocalizations to avoid replaying old sounds on late-join
        if current_time - voc.triggered_at > 5.0 {
            continue;
        }
        audio.play_audio(voc.sound_file.clone(), voc.volume, &voc.position);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (play_interaction_sounds, play_vocalization_sounds)
            .run_if(in_state(UIContextState::InGame)),
    );
}
