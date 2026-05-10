use bevy::prelude::*;
use unaudiospatial_core::emitter::LocalAudioEmitter;
use uncommon_states_core::UIContextState;
use unghost_core::events::GhostAudioMessage;
use unreplicon_core::resources::{AuthorityRole, LocalPlayerRole};

/// Plays authoritative ghost audio broadcasts locally.
fn play_ghost_audio_messages(
    mut reader: MessageReader<GhostAudioMessage>,
    mut audio: LocalAudioEmitter,
    authority: Option<Res<AuthorityRole>>,
) {
    if authority.is_some() {
        for _ in reader.read() {}
        return;
    }

    for msg in reader.read() {
        audio.play_audio(msg.sound_file.clone(), msg.volume, &msg.position);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        play_ghost_audio_messages
            .run_if(resource_exists::<LocalPlayerRole>)
            .run_if(in_state(UIContextState::InGame)),
    );
}
