use bevy::prelude::*;
use unaudiospatial_core::emitter::LocalAudioEmitter;
use uncommon_states_core::UIContextState;
use unghost_core::components::logic::vocalization::GhostVocalization;
use unghost_core::events::GhostAudioMessage;
use unreplicon_core::resources::LocalPlayerRole;

/// Plays authoritative ghost audio broadcasts locally.
fn play_ghost_audio_messages(
    mut reader: MessageReader<GhostAudioMessage>,
    mut audio: LocalAudioEmitter,
) {
    for msg in reader.read() {
        audio.play_audio(msg.sound_file.clone(), msg.volume, &msg.position);
    }
}

/// Plays the remaining replicated vocalization fallback locally when `GhostVocalization`
/// changes. This currently only preserves the death vocalization chain until that
/// sequence is moved to the authoritative broadcast path as well.
fn play_vocalization_sounds(
    q: Query<&GhostVocalization, Changed<GhostVocalization>>,
    mut audio: LocalAudioEmitter,
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
        (play_ghost_audio_messages, play_vocalization_sounds)
            .run_if(resource_exists::<LocalPlayerRole>)
            .run_if(in_state(UIContextState::InGame)),
    );
}
