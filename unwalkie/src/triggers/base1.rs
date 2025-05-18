use bevy::{prelude::*, time::Stopwatch};
use uncore::{
    components::{
        board::position::Position, game_config::GameConfig, ghost_sprite::GhostSprite,
        player_sprite::PlayerSprite,
    },
    difficulty::CurrentDifficulty,
    resources::roomdb::RoomDB,
    states::{AppState, GameState},
};
use ungear::components::playergear::PlayerGear;
use unwalkiecore::{WalkieEvent, WalkiePlay};

/// Reminds the player to pick up equipment if they enter the location without any gear during the tutorial.
/// Only triggers if the player is in the game, not in the truck, and has accessed the truck at least once.
/// Uses a stopwatch to avoid spamming the reminder and only warns within the first minute inside.
fn player_forgot_equipment(
    mut walkie_play: ResMut<WalkiePlay>,
    qp: Query<(&PlayerSprite, &Position, &PlayerGear)>,
    roomdb: Res<RoomDB>,
    gc: Res<GameConfig>,
    mut stopwatch: Local<Stopwatch>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    time: Res<Time>,
) {
    if app_state.get() != &AppState::InGame {
        // We want to play this only when the player is in the game.
        stopwatch.reset();
        return;
    }
    if game_state.get() != &GameState::None {
        // We want to play this only when the player is not in the truck.
        stopwatch.reset();
        return;
    }
    if !walkie_play.truck_accessed {
        // The player didn't had a chance to grab stuff, so don't tell them to.
        stopwatch.reset();
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
        walkie_play.mark(WalkieEvent::GearInVan, time.elapsed_secs_f64());
        return;
    }
    stopwatch.tick(time.delta());
    if stopwatch.elapsed().as_secs_f32() < 1.0 {
        // Wait before reminding the player.
        return;
    }
    if stopwatch.elapsed().as_secs_f32() > 60.0 {
        // Too much time inside the location, we want to warn mainly when it crosses the main door.
        return;
    }
    walkie_play.set(WalkieEvent::GearInVan, time.elapsed_secs_f64());
}

/// Plays a walkie-talkie message at the start of a tutorial mission when the player enters the location.
/// Only triggers in tutorial mode, when the player is in the game and not in the truck.
/// Uses a short delay to avoid playing the message immediately.
fn mission_start_easy(
    mut walkie_play: ResMut<WalkiePlay>,
    difficulty: Res<CurrentDifficulty>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut stopwatch: Local<Stopwatch>,
    time: Res<Time>,
) {
    if difficulty.0.tutorial_chapter.is_none() {
        // Not in tutorial mode, so not an easy difficulty
        stopwatch.reset();
        return;
    }
    if app_state.get() != &AppState::InGame {
        // We want to play this only when the player is in the game.
        stopwatch.reset();
        return;
    }
    if game_state.get() != &GameState::None {
        // We want to play this only when the player is not in the truck.
        stopwatch.reset();
        return;
    }

    stopwatch.tick(time.delta());
    if stopwatch.elapsed().as_secs_f32() < 0.2 {
        // Wait before playing the message.
        return;
    }
    walkie_play.set(WalkieEvent::MissionStartEasy, time.elapsed_secs_f64());
}

/// Warns the player via walkie-talkie when the ghost is close to starting a hunt in the tutorial.
/// Only triggers if the player is inside the location and the ghost's rage is high but not yet hunting.
fn ghost_near_hunt(
    mut walkie_play: ResMut<WalkiePlay>,
    qp: Query<(&PlayerSprite, &Position, &PlayerGear)>,
    roomdb: Res<RoomDB>,
    difficulty: Res<CurrentDifficulty>,
    gc: Res<GameConfig>,
    q_ghost: Query<&GhostSprite>,
    time: Res<Time>,
) {
    if difficulty.0.tutorial_chapter.is_none() {
        // Not in tutorial mode, no need to tell the player.
        return;
    }
    // Find the active player's position
    let Some((player_pos, _player_gear)) = qp.iter().find_map(|(player, pos, gear)| {
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
        // Player is not inside the location, no need to tell them.
        return;
    }
    for ghost in q_ghost.iter() {
        if (ghost.rage > ghost.rage_limit * 0.8) && !ghost.hunt_warning_active && !ghost.hunt_target
        {
            walkie_play.set(WalkieEvent::GhostNearHunt, time.elapsed_secs_f64());
            return;
        }
    }
}

/// Registers the above systems to the Bevy app.
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, player_forgot_equipment)
        .add_systems(Update, mission_start_easy)
        .add_systems(Update, ghost_near_hunt);
}
