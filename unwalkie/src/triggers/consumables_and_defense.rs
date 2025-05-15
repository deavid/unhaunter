use std::any::Any;

use bevy::prelude::*;
use uncore::{
    components::{board::position::Position, player_sprite::PlayerSprite},
    resources::roomdb::RoomDB,
    states::{AppState, GameState},
};
use ungear::components::playergear::PlayerGear;
use ungearitems::components::quartz::QuartzStoneData;
use unwalkiecore::{WalkieEvent, WalkiePlay};

/// Triggers a feedback event when the player's quartz stone cracks, after the hunt is over or player leaves the location.
fn quartz_cracked_feedback(
    mut walkie_play: ResMut<WalkiePlay>,
    qp: Query<(&PlayerSprite, &Position, &PlayerGear)>,
    roomdb: Res<RoomDB>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    time: Res<Time>,
    mut last_cracks: Local<Option<u8>>,
) {
    if app_state.get() != &AppState::InGame || *game_state.get() != GameState::None {
        *last_cracks = None;
        return;
    }
    let Some((_player, pos, gear)) = qp.iter().next() else {
        return;
    };
    let player_bpos = pos.to_board_position();
    if roomdb.room_tiles.get(&player_bpos).is_none() {
        *last_cracks = None;
        return;
    }
    for (g, _) in gear.as_vec() {
        if let Some(quartz) = g
            .data
            .as_ref()
            .and_then(|d| <dyn Any>::downcast_ref::<QuartzStoneData>(d))
        {
            if let Some(prev) = *last_cracks {
                if quartz.cracks > prev && quartz.cracks < 4 {
                    walkie_play.set(WalkieEvent::QuartzCrackedFeedback, time.elapsed_secs_f64());
                }
            }
            *last_cracks = Some(quartz.cracks);
        }
    }
}

/// Triggers a feedback event when the player's quartz stone shatters, after the hunt is over or player leaves the location.
fn quartz_shattered_feedback(
    mut walkie_play: ResMut<WalkiePlay>,
    qp: Query<(&PlayerSprite, &Position, &PlayerGear)>,
    roomdb: Res<RoomDB>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    time: Res<Time>,
    mut shattered: Local<bool>,
) {
    if app_state.get() != &AppState::InGame || *game_state.get() != GameState::None {
        *shattered = false;
        return;
    }
    let Some((_player, pos, gear)) = qp.iter().next() else {
        return;
    };
    let player_bpos = pos.to_board_position();
    if roomdb.room_tiles.get(&player_bpos).is_none() {
        *shattered = false;
        return;
    }
    for (g, _) in gear.as_vec() {
        if let Some(quartz) = g
            .data
            .as_ref()
            .and_then(|d| <dyn Any>::downcast_ref::<QuartzStoneData>(d))
        {
            if quartz.cracks >= 4 && !*shattered {
                walkie_play.set(
                    WalkieEvent::QuartzShatteredFeedback,
                    time.elapsed_secs_f64(),
                );
                *shattered = true;
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, quartz_cracked_feedback)
        .add_systems(Update, quartz_shattered_feedback);
}
