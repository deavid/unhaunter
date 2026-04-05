use bevy::prelude::*;
use uncommon_states_core::UIContextState;

use unaudiobg_core::events::AmbientSoundMuteEvent;
use unaudiobg_core::mute::AmbientMuteController;

use crate::systems::{
    menu_music::{despawn_sound, manage_title_song},
    mute::process_ambient_mute_events,
    spawn::{silence_background_tracks, spawn_background_tracks},
    volume::update_ambient_sound_volumes,
};

pub struct UnhaunterAudioBgPlugin;

impl Plugin for UnhaunterAudioBgPlugin {
    fn build(&self, app: &mut App) {
        // Register the mute event type
        app.add_message::<AmbientSoundMuteEvent>();

        // Initialize mute controller
        app.init_resource::<AmbientMuteController>();

        // Spawn background tracks at startup
        app.add_systems(Startup, spawn_background_tracks);

        // Silence tracks when exiting game
        app.add_systems(OnExit(UIContextState::InGame), silence_background_tracks);

        // Update systems
        app.add_systems(
            Update,
            (
                process_ambient_mute_events.run_if(in_state(UIContextState::InGame)),
                update_ambient_sound_volumes.run_if(in_state(UIContextState::InGame)),
            ),
        );

        // Menu music systems always run
        app.add_systems(Update, (manage_title_song, despawn_sound));
    }
}
