use bevy::{audio::Volume, prelude::*, time::Stopwatch};
use bevy_persistent::Persistent;
use rand::seq::IndexedRandom;
use uncore::{
    components::{
        board::position::Position, game_config::GameConfig, game_ui::WalkieText,
        player_sprite::PlayerSprite,
    },
    difficulty::CurrentDifficulty,
    events::{loadlevel::LevelReadyEvent, walkie::WalkieEvent},
    random_seed,
    resources::{
        roomdb::RoomDB,
        walkie::{WalkiePlay, WalkieSoundState},
    },
    states::{AppState, GameState},
};
use ungear::components::playergear::PlayerGear;
use unsettings::audio::AudioSettings;

fn player_forgot_equipment(
    mut walkie_play: ResMut<WalkiePlay>,
    qp: Query<(&PlayerSprite, &Position, &PlayerGear)>,
    roomdb: Res<RoomDB>,
    difficulty: Res<CurrentDifficulty>,
    gc: Res<GameConfig>,
    mut stopwatch: Local<Stopwatch>,
    time: Res<Time>,
) {
    if difficulty.0.tutorial_chapter.is_none() {
        // Not in tutorial mode, no need to remind the player.
        return;
    }
    if !walkie_play.truck_accessed {
        // The player didn't had a chance to grab stuff, so don't tell them to.
        return;
    }
    // Find the active player's position
    let Some((player_pos, player_gear)) = qp.iter().find_map(|(player, pos, gear)| {
        if player.id == gc.player_id {
            Some((*pos, gear))
        } else {
            None
        }
    }) else {
        return;
    };
    let player_bpos = player_pos.to_board_position();

    if roomdb.room_tiles.get(&player_bpos).is_none() {
        // Player is not inside the location, no need to remind them.
        stopwatch.reset();
        return;
    }
    if !player_gear.empty_right_handed() {
        // Player has an item, no need to remind them.
        walkie_play.mark(WalkieEvent::GearInVan);
        return;
    }
    stopwatch.tick(time.delta());
    if stopwatch.elapsed().as_secs_f32() < 1.0 {
        // Wait before reminding the player.
        return;
    }
    stopwatch.reset();
    walkie_play.set(WalkieEvent::GearInVan);
}

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
    mut walkie_play: ResMut<WalkiePlay>,
    q_sound_state: Query<&WalkieSoundState>,
    mut qt: Query<&mut Text, With<WalkieText>>,
) {
    let mut rng = random_seed::rng();

    let Some(walkie_event) = walkie_play.event.clone() else {
        return;
    };
    if q_sound_state.iter().count() > 0 {
        // Already playing a sound
        return;
    }
    let mut walkie_volume = 1.0;
    let mut talking_sound_file = "sounds/radio-on-zzt.ogg";
    let new_state = match walkie_play.state {
        None => Some(WalkieSoundState::Intro),
        Some(WalkieSoundState::Intro) => {
            let files = walkie_event.sound_file_list();
            talking_sound_file = files
                .choose(&mut rng)
                .copied()
                .unwrap_or("sounds/radio-on-zzt.ogg");
            Some(WalkieSoundState::Talking)
        }
        Some(WalkieSoundState::Talking) => Some(WalkieSoundState::Outro),
        Some(WalkieSoundState::Outro) => None,
    };

    for mut text in qt.iter_mut() {
        text.0 = match new_state {
            Some(WalkieSoundState::Intro) => "**bzzrt**".to_string(),
            Some(WalkieSoundState::Talking) => {
                format!(
                    "{}  {}",
                    text.0,
                    WalkieEvent::voice_text(talking_sound_file)
                )
            }
            Some(WalkieSoundState::Outro) => format!("{} **bzzrt**", text.0),
            None => "".to_string(),
        };
    }

    walkie_play.state = new_state.clone();
    if new_state.is_none() {
        // Done playing
        walkie_play.event = None;
        return;
    }

    let new_state = new_state.unwrap();

    let sound_file = match new_state {
        WalkieSoundState::Intro => "sounds/radio-on-zzt.ogg",
        WalkieSoundState::Talking => {
            walkie_volume = 0.5;
            talking_sound_file
        }
        WalkieSoundState::Outro => "sounds/radio-off-zzt.ogg",
    };

    commands
        .spawn(AudioPlayer::new(asset_server.load(sound_file)))
        .insert(PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: Volume::new(
                walkie_volume
                    * audio_settings.volume_voice_chat.as_f32()
                    * audio_settings.volume_master.as_f32(),
            ),
            speed: 1.0,
            paused: false,
            spatial: false,
            spatial_scale: None,
        })
        .insert(new_state);
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, player_forgot_equipment)
        .add_systems(Update, walkie_talk)
        .add_systems(Update, on_game_load)
        .add_systems(Update, state_tracking);
}
