use bevy::{audio::Volume, prelude::*};
use bevy_persistent::Persistent;
use rand::seq::IndexedRandom;
use uncore::{
    components::{board::position::Position, game_config::GameConfig, player_sprite::PlayerSprite},
    difficulty::CurrentDifficulty,
    events::{loadlevel::LevelReadyEvent, walkie::WalkieEvent},
    random_seed,
    resources::{
        roomdb::RoomDB,
        walkie::{WalkiePlay, WalkieSoundState},
    },
};
use ungear::components::playergear::PlayerGear;
use unsettings::audio::AudioSettings;

pub fn player_forgot_equipment(
    mut walkie_play: ResMut<WalkiePlay>,
    qp: Query<(&PlayerSprite, &Position, &PlayerGear)>,
    roomdb: Res<RoomDB>,
    difficulty: Res<CurrentDifficulty>,
    gc: Res<GameConfig>,
) {
    if difficulty.0.tutorial_chapter.is_none() {
        // Not in tutorial mode, no need to remind the player.
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
        return;
    }
    if !player_gear.empty_right_handed() {
        // Player has an item, no need to remind them.
        walkie_play.mark(WalkieEvent::GearInVan);
        return;
    }

    walkie_play.set(WalkieEvent::GearInVan);
}

pub fn on_game_load(
    mut ev_level_ready: EventReader<LevelReadyEvent>,
    mut walkie_play: ResMut<WalkiePlay>,
) {
    for _ in ev_level_ready.read() {
        // Reset the walkie play state
        walkie_play.reset();
    }
}

pub fn walkie_talk(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<Persistent<AudioSettings>>,
    mut walkie_play: ResMut<WalkiePlay>,
    q_sound_state: Query<&WalkieSoundState>,
) {
    let Some(walkie_event) = walkie_play.event.clone() else {
        return;
    };
    if q_sound_state.iter().count() > 0 {
        // Already playing a sound
        return;
    }
    let mut walkie_volume = 1.0;
    let new_state = match walkie_play.state {
        None => WalkieSoundState::Intro,
        Some(WalkieSoundState::Intro) => WalkieSoundState::Talking,
        Some(WalkieSoundState::Talking) => WalkieSoundState::Outro,
        Some(WalkieSoundState::Outro) => {
            walkie_play.event = None;
            walkie_play.state = None;
            return;
        }
    };
    walkie_play.state = Some(new_state.clone());
    let mut rng = random_seed::rng();

    let sound_file = match new_state {
        WalkieSoundState::Intro => "sounds/radio-on-zzt.ogg",
        WalkieSoundState::Talking => {
            walkie_volume = 0.5;
            let files = walkie_event.sound_file_list();
            files
                .choose(&mut rng)
                .copied()
                .unwrap_or("sounds/radio-on-zzt.ogg")
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
