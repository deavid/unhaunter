use bevy::{prelude::*, time::Stopwatch};
use bevy_kira_audio::Audio; // Explicit import to resolve naming conflicts
use bevy_kira_audio::prelude::*;
use bevy_persistent::Persistent;
use rand::seq::IndexedRandom;
use uncore::{
    components::game_ui::WalkieText,
    events::{hint::OnScreenHintEvent, loadlevel::LevelReadyEvent},
    random_seed,
    states::{AppState, GameState},
};
use unsettings::audio::AudioSettings;
use unwalkie_types::VoiceLineData;
use unwalkiecore::{WalkiePlay, WalkieSoundState, WalkieTalkingEvent};

fn on_game_load(
    mut ev_level_ready: EventReader<LevelReadyEvent>,
    mut walkie_play: ResMut<WalkiePlay>,
) {
    for _ in ev_level_ready.read() {
        // Reset the walkie play state
        walkie_play.reset();
    }
}

fn state_tracking(
    mut walkie_play: ResMut<WalkiePlay>,
    _app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
) {
    if *game_state.get() == GameState::Truck {
        walkie_play.truck_accessed = true;
    }
}

fn walkie_talk(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<Persistent<AudioSettings>>,
    audio: Res<Audio>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut hint_event_writer: EventWriter<OnScreenHintEvent>,
    mut walkie_talking_writer: EventWriter<WalkieTalkingEvent>,
    q_sound_state: Query<(Entity, &WalkieSoundState)>,
    mut qt: Query<&mut Text, With<WalkieText>>,
    mut stopwatch: Local<Stopwatch>,
    time: Res<Time>,
) {
    let mut rng = random_seed::rng();
    walkie_play.priority_bar /= 1.2;

    let Some(walkie_event) = walkie_play.event.clone() else {
        stopwatch.reset();
        return;
    };
    if q_sound_state.iter().count() > 0 {
        // Already playing a sound
        if walkie_play.urgent_pending {
            // Stop all sounds, clean up the state.
            walkie_play.event = None;
            walkie_play.state = None;
            walkie_play.current_voice_line = None;
            walkie_play.urgent_pending = false;
            for mut text in qt.iter_mut() {
                text.0 = "".to_string();
            }
            stopwatch.reset();
            // Also despawn the sound
            for (entity, _sound_state) in q_sound_state.iter() {
                commands.entity(entity).despawn();
            }
        }
        return;
    }
    let mut walkie_volume = 1.0;

    let new_state = match walkie_play.state {
        None => Some(WalkieSoundState::Intro),
        Some(WalkieSoundState::Intro) => {
            let voice_lines: Vec<VoiceLineData> = walkie_event.sound_file_list();
            if let Some(chosen_line) = voice_lines.choose(&mut rng).cloned() {
                walkie_play.current_voice_line = Some(chosen_line);
            } else {
                walkie_play.current_voice_line = Some(VoiceLineData {
                    ogg_path: "sounds/radio-on-zzt.ogg".to_string(),
                    subtitle_text: "[NO SUBTITLE AVAILABLE]".to_string(),
                    tags: vec![],
                    length_seconds: 2,
                });
            }

            // Fire WalkieTalkingEvent when transitioning to the Talking state
            walkie_talking_writer.write(WalkieTalkingEvent {
                event: walkie_event.clone(),
            });

            Some(WalkieSoundState::Talking)
        }
        Some(WalkieSoundState::Talking) => {
            let hint_text = walkie_event.get_on_screen_actionable_hint_text();
            if !hint_text.is_empty() {
                let saved_count = walkie_play
                    .other_mission_event_count
                    .get(&walkie_event)
                    .copied()
                    .unwrap_or_default();
                use rand::Rng;
                let mut rng = random_seed::rng();
                let dice = rng.random_range(0..=saved_count.pow(2));
                info!(
                    "hint dice: {:?}: {}/{}",
                    walkie_event,
                    dice,
                    saved_count.pow(2)
                );
                if dice < 8 {
                    hint_event_writer.write(OnScreenHintEvent {
                        hint_text: hint_text.to_string(),
                    });
                }
            }

            Some(WalkieSoundState::Outro)
        }
        Some(WalkieSoundState::Outro) => {
            stopwatch.tick(time.delta());
            if stopwatch.elapsed().as_secs_f32() < 2.0 {
                return;
            }
            None
        }
    };
    stopwatch.reset();

    for mut text in qt.iter_mut() {
        text.0 = match new_state {
            Some(WalkieSoundState::Intro) => "**bzzrt**".to_string(),
            Some(WalkieSoundState::Talking) => {
                if let Some(voice_line) = &walkie_play.current_voice_line {
                    format!("{}  {}", text.0, voice_line.subtitle_text)
                } else {
                    format!("{}  [ERROR: Missing subtitle]", text.0)
                }
            }
            Some(WalkieSoundState::Outro) => format!("{} **bzzrt**", text.0),
            None => "".to_string(),
        };
    }

    walkie_play.state = new_state.clone();
    if new_state.is_none() {
        // When walkie ends, send an OnScreenHintEvent with on_completion=true
        walkie_play.event = None;
        walkie_play.current_voice_line = None;
        walkie_play.last_message_time = time.elapsed_secs_f64();
        return;
    }

    let new_state_unwrapped = new_state.unwrap();

    let sound_file = match new_state_unwrapped {
        WalkieSoundState::Intro => "sounds/radio-on-zzt.ogg",
        WalkieSoundState::Talking => {
            walkie_volume = 0.2;
            if let Some(voice_line) = &walkie_play.current_voice_line {
                &voice_line.ogg_path
            } else {
                "sounds/radio-on-zzt.ogg"
            }
        }
        WalkieSoundState::Outro => "sounds/radio-off-zzt.ogg",
    };

    // Play audio using bevy_kira_audio
    let audio_handle = asset_server.load(sound_file);
    let final_volume = walkie_volume
        * audio_settings.volume_voice_chat.as_f32()
        * audio_settings.volume_master.as_f32();

    audio.play(audio_handle).with_volume(final_volume as f64);

    // Spawn a marker entity to track the sound state
    commands.spawn(new_state_unwrapped);
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, walkie_talk)
        .add_systems(Update, on_game_load)
        .add_systems(Update, state_tracking);
}
