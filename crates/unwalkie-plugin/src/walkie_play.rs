use bevy::{prelude::*, time::Stopwatch};
use unaudiospatial_core::components::{AudioCategory, FlatAudio};
use uncommon_app_core::random_seed;
use unmission_core::events::LevelReadyEvent;
use unplayer_core::components::MainPlayer;
use untruck_core::components::in_truck::InTruck;
use unwalkie_core::components::WalkieText;
use unwalkie_core::events::hint::OnScreenHintEvent;
use unwalkie_core::events::walkie_types::WalkieTalkingEvent;
use unwalkie_core::resources::{WalkiePlay, WalkieSoundState};

fn on_game_load(
    mut ev_level_ready: MessageReader<LevelReadyEvent>,
    mut walkie_play: ResMut<WalkiePlay>,
) {
    for _ in ev_level_ready.read() {
        // Reset the walkie play state
        walkie_play.reset();
    }
}

fn state_tracking(
    mut walkie_play: ResMut<WalkiePlay>,
    q_in_truck: Query<(), (With<MainPlayer>, With<InTruck>)>,
) {
    if !q_in_truck.is_empty() {
        walkie_play.truck_accessed = true;
    }
}

fn walkie_talk(
    mut commands: Commands,
    mut walkie_play: ResMut<WalkiePlay>,
    mut hint_event_writer: MessageWriter<OnScreenHintEvent>,
    mut walkie_talking_writer: MessageWriter<WalkieTalkingEvent>,
    q_sound_state: Query<(Entity, &WalkieSoundState)>,
    mut qt: Query<(&mut Text, &mut Visibility), With<WalkieText>>,
    mut stopwatch: Local<Stopwatch>,
    time: Res<Time>,
) {
    walkie_play.priority_bar /= 1.2;

    let Some(walkie_event) = walkie_play.event.clone() else {
        stopwatch.reset();
        return;
    };

    if q_sound_state.iter().count() > 0 {
        // Already playing a sound
        if walkie_play.urgent_pending {
            // Stop all sounds, clean up the state.

            // Since the event was interrupted, we want to allow it to be played again later.
            // We revert the stats for this event so it doesn't count as "consumed".
            if let Some(interrupted_event) = walkie_play.event.clone() {
                let mut remove = false;
                if let Some(stats) = walkie_play.played_events.get_mut(&interrupted_event) {
                    if stats.count > 1 {
                        stats.count -= 1;
                        // Allow immediate retry once the channel is free
                        stats.last_played = 0.0;
                    } else {
                        remove = true;
                    }
                }
                if remove {
                    walkie_play.played_events.remove(&interrupted_event);
                }
            }

            walkie_play.event = None;
            walkie_play.state = None;
            walkie_play.current_voice_line = None;
            walkie_play.urgent_pending = false;
            for (mut text, mut vis) in qt.iter_mut() {
                text.0 = "".to_string();
                *vis = Visibility::Hidden;
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
    let state_changed = walkie_play.tick_state();

    if state_changed {
        if let Some(WalkieSoundState::Talking) = &walkie_play.state {
            // Fire WalkieTalkingEvent when transitioning to the Talking state
            walkie_talking_writer.write(WalkieTalkingEvent {
                event: walkie_event.clone(),
            });

            let hint_text = walkie_event.get_on_screen_actionable_hint_text();
            if !hint_text.is_empty() {
                let saved_count = walkie_play
                    .other_mission_event_count
                    .get(&walkie_event)
                    .copied()
                    .unwrap_or_default();
                use rand::prelude::*;
                let mut rng = random_seed::rng();
                let dice = rng.random_range(0..=saved_count.pow(2));
                debug!(
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
        }
    } else if let Some(WalkieSoundState::Outro) = &walkie_play.state {
        stopwatch.tick(time.delta());
        if stopwatch.elapsed().as_secs_f32() > 2.0 {
            walkie_play.event = None;
            walkie_play.state = None;
            walkie_play.current_voice_line = None;
            walkie_play.last_message_time = time.elapsed_secs_f64();
            for (mut text, mut vis) in qt.iter_mut() {
                text.0 = "".to_string();
                *vis = Visibility::Hidden;
            }
            return;
        }
        return; // Still waiting for Outro to finish
    }

    if !state_changed {
        return; // Wait until sounds finish before doing anything else
    }

    let new_state = walkie_play.state.clone();
    stopwatch.reset();

    for (mut text, mut vis) in qt.iter_mut() {
        if new_state.is_some() {
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
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

    if new_state != walkie_play.state {
        debug!(
            "WALKIE_PLAY: state transition {:?} -> {:?}",
            walkie_play.state, new_state
        );
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
        WalkieSoundState::Intro => "sounds/radio-on-zzt.ogg".to_string(),
        WalkieSoundState::Talking => {
            walkie_volume = 0.2;
            if let Some(voice_line) = &walkie_play.current_voice_line {
                voice_line.ogg_path.clone()
            } else {
                "sounds/radio-on-zzt.ogg".to_string()
            }
        }
        WalkieSoundState::Outro => "sounds/radio-off-zzt.ogg".to_string(),
    };

    commands.spawn((
        FlatAudio {
            sound_file,
            volume_multiplier: walkie_volume,
            category: AudioCategory::VoiceChat,
        },
        new_state_unwrapped,
    ));
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, walkie_talk)
        .add_systems(Update, on_game_load)
        .add_systems(Update, state_tracking);
}
