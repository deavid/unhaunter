use std::any::Any;

use bevy::prelude::*;
use uncore::{
    components::{
        board::position::Position, ghost_sprite::GhostSprite, player_sprite::PlayerSprite,
    },
    difficulty::CurrentDifficulty,
    resources::roomdb::RoomDB,
    states::{AppState, GameState},
    types::{gear_kind::GearKind, manual::ManualChapterIndex},
};
use ungear::components::playergear::PlayerGear;
use ungearitems::{components::quartz::QuartzStoneData, prelude::SageBundleData};
use untruck::truckgear::TruckGear;
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

// TODO (David): Add `times_hunted_this_mission: u32` to `GhostSprite` struct
// in `uncore/src/components/ghost_sprite.rs` and ensure it's incremented
// when a hunt truly begins in `unghost/src/ghost.rs`. Initialize to 0.

fn trigger_quartz_unused_in_relevant_situation_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<&PlayerGear, With<PlayerSprite>>,
    ghost_query: Query<&GhostSprite>,
    difficulty: Res<CurrentDifficulty>,
    truck_gear: Res<TruckGear>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        return;
    }

    // 2. Chapter Check: Only trigger for Chapter 5 or non-tutorial difficulties
    let current_chapter_index = difficulty
        .0
        .tutorial_chapter
        .map(|c| c.index())
        .unwrap_or(usize::MAX); // usize::MAX if not tutorial
    if current_chapter_index < ManualChapterIndex::Chapter5.index() {
        // If it's a tutorial chapter AND it's before Chapter 5, exit.
        // Non-tutorial difficulties (where tutorial_chapter is None, so current_chapter_index is usize::MAX) will pass this.
        return;
    }

    let Ok(player_gear) = player_query.get_single() else {
        return;
    };
    let Ok(ghost_sprite) = ghost_query.get_single() else {
        return;
    };

    // 4. First Hunt Check
    // Assuming `times_hunted_this_mission` is added to GhostSprite
    if ghost_sprite.times_hunted_this_mission == 0 {
        return; // Hint is for after experiencing at least one hunt
    }

    // 5. Relevant Situation Check (Hunt Likely)
    let is_hunt_likely =
        ghost_sprite.hunt_warning_active || (ghost_sprite.rage > ghost_sprite.rage_limit * 0.70); // 70% rage threshold
    if !is_hunt_likely {
        return;
    }

    // 6. Check Player Inventory for Quartz
    let player_has_quartz = player_gear.as_vec().iter().any(|(gear, _epos)| {
        if gear.kind == GearKind::QuartzStone {
            // Further check if it's not shattered (assuming QuartzStoneData is accessible)
            // For now, just checking the kind. If it's shattered, it's effectively not "had".
            // This check can be refined if QuartzStoneData stores a `shattered` bool.
            // For simplicity now, if they have the *item kind*, we assume they *have* quartz.
            // The `QuartzShatteredFeedback` event handles telling them it's broken.
            true
        } else {
            false
        }
    });
    if player_has_quartz {
        return; // Player already has quartz, no need for this hint
    }

    // 7. Check Truck Inventory for Quartz
    let truck_has_quartz = truck_gear
        .inventory
        .iter()
        .any(|gear| gear.kind == GearKind::QuartzStone);
    if !truck_has_quartz {
        return; // Quartz isn't even available in the truck
    }

    // 8. Trigger Event: All conditions met
    walkie_play.set(
        WalkieEvent::QuartzUnusedInRelevantSituation,
        time.elapsed_secs_f64(),
    );
}

fn trigger_sage_unused_in_relevant_situation_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<&PlayerGear, With<PlayerSprite>>,
    ghost_query: Query<&GhostSprite>,
    difficulty: Res<CurrentDifficulty>,
    truck_gear: Res<TruckGear>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        return;
    }

    // 2. Chapter Check: Only trigger for Chapter 5 or non-tutorial difficulties
    let current_chapter_index = difficulty
        .0
        .tutorial_chapter
        .map(|c| c.index())
        .unwrap_or(usize::MAX);
    if current_chapter_index < ManualChapterIndex::Chapter5.index() {
        return;
    }

    let Ok(player_gear) = player_query.get_single() else {
        return;
    };
    let Ok(ghost_sprite) = ghost_query.get_single() else {
        return;
    };

    // 4. First Hunt Check
    if ghost_sprite.times_hunted_this_mission == 0 {
        return;
    }

    // 5. Relevant Situation Check (Hunt Likely)
    let is_hunt_likely =
        ghost_sprite.hunt_warning_active || (ghost_sprite.rage > ghost_sprite.rage_limit * 0.70);
    if !is_hunt_likely {
        return;
    }

    // 6. Check Player Inventory for Unconsumed Sage
    let player_has_unconsumed_sage = player_gear.as_vec().iter().any(|(gear, _epos)| {
        if gear.kind == GearKind::SageBundle {
            if let Some(sage_data_dyn) = gear.data.as_ref() {
                if let Some(sage_data) = <dyn Any>::downcast_ref::<SageBundleData>(sage_data_dyn) {
                    return !sage_data.consumed; // Player has sage and it's not consumed
                }
            }
        }
        false
    });

    if player_has_unconsumed_sage {
        return; // Player already has usable sage
    }

    // 7. Check Truck Inventory for Sage
    let truck_has_sage = truck_gear
        .inventory
        .iter()
        .any(|gear| gear.kind == GearKind::SageBundle);
    if !truck_has_sage {
        return; // Sage isn't even available in the truck
    }

    // 8. Trigger Event: All conditions met
    walkie_play.set(
        WalkieEvent::SageUnusedInRelevantSituation,
        time.elapsed_secs_f64(),
    );
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, quartz_cracked_feedback);
    app.add_systems(Update, quartz_shattered_feedback);
    app.add_systems(Update, trigger_quartz_unused_in_relevant_situation_system);
    app.add_systems(Update, trigger_sage_unused_in_relevant_situation_system);
}
