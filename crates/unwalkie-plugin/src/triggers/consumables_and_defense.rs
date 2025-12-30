use bevy::prelude::*;
use uncore_board::resources::roomdb::RoomDB;
use uncore_resources::states::{AppState, GameState};
use undifficulty_core::CurrentDifficulty;
use undifficulty_core::ManualChapterIndex;
use ungear_core::GearKind;
use ungear_core::components::playergear::PlayerGear;
use ungearitems_core::{components::quartz::QuartzStoneData, prelude::SageBundleData};
use unghost_core::components::GhostSprite;
use unplayer_core::components::PlayerSprite;
use unspatial_core::Position;
use untruck_core::truckgear::TruckGear;
use unwalkie_core::{WalkieEvent, WalkiePlay};

/// Triggers a feedback event when the player's quartz stone cracks, after the hunt is over or player leaves the location.
fn quartz_cracked_feedback(
    mut walkie_play: ResMut<WalkiePlay>,
    qp: Query<(&PlayerSprite, &Position, &PlayerGear)>,
    q_quartz: Query<&QuartzStoneData>,
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
    let gear_iter = gear
        .left_hand
        .iter()
        .chain(gear.right_hand.iter())
        .chain(gear.inventory.iter());

    for entity in gear_iter {
        if let Ok(quartz) = q_quartz.get(*entity) {
            if let Some(prev) = *last_cracks
                && quartz.cracks > prev
                && quartz.cracks < 4
            {
                // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
                walkie_play.set(WalkieEvent::QuartzCrackedFeedback, time.elapsed_secs_f64());
            }
            *last_cracks = Some(quartz.cracks);
        }
    }
}

/// Triggers a feedback event when the player's quartz stone shatters, after the hunt is over or player leaves the location.
fn quartz_shattered_feedback(
    mut walkie_play: ResMut<WalkiePlay>,
    qp: Query<(&PlayerSprite, &Position, &PlayerGear)>,
    q_quartz: Query<&QuartzStoneData>,
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
    let gear_iter = gear
        .left_hand
        .iter()
        .chain(gear.right_hand.iter())
        .chain(gear.inventory.iter());

    for entity in gear_iter {
        if let Ok(quartz) = q_quartz.get(*entity)
            && quartz.cracks >= 4
            && !*shattered
        {
            // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
            walkie_play.set(
                WalkieEvent::QuartzShatteredFeedback,
                time.elapsed_secs_f64(),
            );
            *shattered = true;
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
    truck_gear: Option<Res<TruckGear>>,
    q_gear: Query<&GearKind>,
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

    let Ok(player_gear) = player_query.single() else {
        return;
    };
    let Ok(ghost_sprite) = ghost_query.single() else {
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
    let check_gear = |entity: Entity| -> bool {
        if let Ok(kind) = q_gear.get(entity) {
            *kind == GearKind::QuartzStone
        } else {
            false
        }
    };

    let player_has_quartz = player_gear.left_hand.map(check_gear).unwrap_or(false)
        || player_gear.right_hand.map(check_gear).unwrap_or(false)
        || player_gear.inventory.iter().any(|&e| check_gear(e));

    if player_has_quartz {
        return; // Player already has quartz, no need for this hint
    }
    let truck_gear = match truck_gear {
        Some(gear) => gear,
        None => return, // No truck gear available, exit early
    };
    // 7. Check Truck Inventory for Quartz
    let truck_has_quartz = truck_gear.inventory.iter().any(|&gear| check_gear(gear));
    if !truck_has_quartz {
        return; // Quartz isn't even available in the truck
    }
    // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
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
    truck_gear: Option<Res<TruckGear>>,
    q_gear: Query<&GearKind>,
    q_sage: Query<&SageBundleData>,
) {
    let truck_gear = match truck_gear {
        Some(gear) => gear,
        None => return, // No truck gear available, exit early
    };
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

    let Ok(player_gear) = player_query.single() else {
        return;
    };
    let Ok(ghost_sprite) = ghost_query.single() else {
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
    let check_sage = |entity: Entity| -> bool {
        if let Ok(kind) = q_gear.get(entity)
            && *kind == GearKind::SageBundle
            && let Ok(sage_data) = q_sage.get(entity)
        {
            return !sage_data.consumed;
        }
        false
    };

    let player_has_unconsumed_sage = player_gear.left_hand.map(check_sage).unwrap_or(false)
        || player_gear.right_hand.map(check_sage).unwrap_or(false)
        || player_gear.inventory.iter().any(|&e| check_sage(e));

    if player_has_unconsumed_sage {
        return; // Player already has usable sage
    }

    // 7. Check Truck Inventory for Sage
    let check_gear = |entity: Entity| -> bool {
        if let Ok(kind) = q_gear.get(entity) {
            *kind == GearKind::SageBundle
        } else {
            false
        }
    };
    let truck_has_sage = truck_gear.inventory.iter().any(|&gear| check_gear(gear));
    if !truck_has_sage {
        return; // Sage isn't even available in the truck
    }
    // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
    // 8. Trigger Event: All conditions met
    walkie_play.set(
        WalkieEvent::SageUnusedInRelevantSituation,
        time.elapsed_secs_f64(),
    );
}

const MIN_EFFECTIVE_SAGE_CALM_INCREASE: f32 = 5.0; // Seconds of calm_time added to be "effective"
const SAGE_TRACKING_TIMEOUT_SECONDS: f32 = 15.0; // A bit longer than sage burn time

#[derive(Default, Clone, Debug)]
struct SageEffectivenessTracker {
    player_entity_id: Option<Entity>, // Track which player's sage
    sage_activated_game_time: f32,
    initial_ghost_calm_time_secs: f32,
    is_tracking_this_sage_burn: bool,
}

fn trigger_sage_activated_ineffectively_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(Entity, &PlayerGear), With<PlayerSprite>>, // Added Entity to ID player
    ghost_query: Query<&GhostSprite>,
    difficulty: Res<CurrentDifficulty>,
    mut tracker: Local<SageEffectivenessTracker>,
    q_gear: Query<&GearKind>,
    q_sage: Query<&SageBundleData>,
) {
    // 1. System Run Condition & Chapter Check & Reset conditions
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        if tracker.is_tracking_this_sage_burn {
            *tracker = SageEffectivenessTracker::default();
        }
        return;
    }
    let current_chapter_index = difficulty
        .0
        .tutorial_chapter
        .map(|c| c.index())
        .unwrap_or(usize::MAX);
    if current_chapter_index < ManualChapterIndex::Chapter5.index() {
        if tracker.is_tracking_this_sage_burn {
            *tracker = SageEffectivenessTracker::default();
        }
        return;
    }

    // 2. Get Player & Ghost Info
    let Ok((player_entity, player_gear)) = player_query.single() else {
        // Assuming single player
        if tracker.is_tracking_this_sage_burn {
            *tracker = SageEffectivenessTracker::default();
        }
        return;
    };
    let Ok(ghost_sprite) = ghost_query.single() else {
        // Assuming single ghost
        if tracker.is_tracking_this_sage_burn {
            *tracker = SageEffectivenessTracker::default();
        }
        return;
    };

    // 3. Find Sage in Player's Gear
    let mut current_sage_data: Option<&SageBundleData> = None;
    let gear_iter = player_gear
        .left_hand
        .iter()
        .chain(player_gear.right_hand.iter())
        .chain(player_gear.inventory.iter());

    for entity in gear_iter {
        if let Ok(kind) = q_gear.get(*entity)
            && *kind == GearKind::SageBundle
            && let Ok(sage_data) = q_sage.get(*entity)
        {
            current_sage_data = Some(sage_data);
            break;
        }
    }

    let Some(sage_data) = current_sage_data else {
        // Player is not holding sage, or it's not the right type somehow.
        // If we were tracking, and now they don't have sage (e.g. dropped), reset.
        if tracker.is_tracking_this_sage_burn {
            *tracker = SageEffectivenessTracker::default();
        }
        return;
    };

    // 4. Manage Tracker State
    if sage_data.is_active && !sage_data.consumed {
        // Sage is currently burning
        if !tracker.is_tracking_this_sage_burn || tracker.player_entity_id != Some(player_entity) {
            // Start tracking this new burn, or re-track if player changed
            *tracker = SageEffectivenessTracker {
                player_entity_id: Some(player_entity),
                sage_activated_game_time: time.elapsed_secs(),
                initial_ghost_calm_time_secs: ghost_sprite.calm_time_secs,
                is_tracking_this_sage_burn: true,
            };
        }
        // else, already tracking this burn, just let it continue
    } else {
        // Sage is NOT currently active (either consumed or not lit)
        if tracker.is_tracking_this_sage_burn && tracker.player_entity_id == Some(player_entity) {
            // Sage was being tracked for this player, and now it's no longer active.
            // This means it was either consumed or deactivated (e.g. player dropped/stowed it).
            // If it was consumed, this is when we check effectiveness.
            if sage_data.consumed {
                // Check the consumed flag
                let calm_increase =
                    ghost_sprite.calm_time_secs - tracker.initial_ghost_calm_time_secs;
                if calm_increase < MIN_EFFECTIVE_SAGE_CALM_INCREASE {
                    // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
                    walkie_play.set(
                        WalkieEvent::SageActivatedIneffectively,
                        time.elapsed_secs_f64(),
                    );
                }
            }
            // Whether consumed or just deactivated, stop tracking this specific burn.
            *tracker = SageEffectivenessTracker::default();
        }
        // else, wasn't tracking or tracking for a different player, do nothing.
    }

    // Timeout for safety: if sage has been "active" for too long in tracker, reset.
    if tracker.is_tracking_this_sage_burn
        && time.elapsed_secs() - tracker.sage_activated_game_time > SAGE_TRACKING_TIMEOUT_SECONDS
    {
        // info!("Sage tracking timed out for player {:?}. Resetting.", tracker.player_entity_id);
        *tracker = SageEffectivenessTracker::default();
    }
}

#[derive(Default, Clone, Debug)]
enum HuntPhaseForSageCheck {
    #[default]
    NotInHunt,
    InHunt {
        sage_was_activated_during_this_hunt: bool,
    },
}

#[derive(Default, Clone, Debug, Resource)] // Make it a resource for easier reset
struct HuntSageUsageTracker {
    phase: HuntPhaseForSageCheck,
}

// System to reset the tracker when a new mission starts or player leaves InGame
fn reset_hunt_sage_tracker_on_mission_change(
    mut tracker: ResMut<HuntSageUsageTracker>,
    app_state: Res<State<AppState>>,
    mut last_app_state: Local<Option<AppState>>, // Track previous app state
) {
    let current_app_state = *app_state.get();
    if *last_app_state != Some(current_app_state) {
        // If app state changed or it's the first run
        if current_app_state != AppState::InGame
            || (last_app_state.is_some()
                && last_app_state.unwrap() != AppState::InGame
                && current_app_state == AppState::InGame)
        {
            // If we are NOT in game, OR if we JUST entered InGame (new mission)
            if !matches!(tracker.phase, HuntPhaseForSageCheck::NotInHunt) {
                // info!("Resetting HuntSageUsageTracker due to state change or new mission.");
                *tracker = HuntSageUsageTracker::default();
            }
        }
    }
    *last_app_state = Some(current_app_state);
}

fn trigger_sage_unused_defensively_during_hunt_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<&PlayerGear, With<PlayerSprite>>,
    ghost_query: Query<&GhostSprite>,
    difficulty: Res<CurrentDifficulty>,
    mut tracker: ResMut<HuntSageUsageTracker>, // Use ResMut for the tracker
    q_gear: Query<&GearKind>,
    q_sage: Query<&SageBundleData>,
) {
    // 1. System Run Condition & Chapter Check
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        // Tracker reset is handled by `reset_hunt_sage_tracker_on_mission_change`
        return;
    }
    let current_chapter_index = difficulty
        .0
        .tutorial_chapter
        .map(|c| c.index())
        .unwrap_or(usize::MAX);
    if current_chapter_index < ManualChapterIndex::Chapter5.index() {
        return;
    }

    // 2. Get Player & Ghost Info
    let Ok(player_gear) = player_query.single() else {
        return;
    };
    let Ok(ghost_sprite) = ghost_query.single() else {
        // If ghost is gone, and we were in a hunt, treat hunt as ended.
        if matches!(tracker.phase, HuntPhaseForSageCheck::InHunt { .. }) {
            // info!("Ghost despawned during tracked hunt, resetting HuntSageUsageTracker.");
            *tracker = HuntSageUsageTracker::default();
        }
        return;
    };

    // 4. Monitor Hunt State & Sage Usage
    let current_ghost_is_hunting = ghost_sprite.hunting > 0.1; // Threshold for "actively hunting"

    match &mut tracker.phase {
        HuntPhaseForSageCheck::NotInHunt => {
            if current_ghost_is_hunting {
                // Hunt just started
                tracker.phase = HuntPhaseForSageCheck::InHunt {
                    sage_was_activated_during_this_hunt: false,
                };
                // info!("Hunt started. Tracking sage usage.");
            }
        }
        HuntPhaseForSageCheck::InHunt {
            sage_was_activated_during_this_hunt,
        } => {
            if !current_ghost_is_hunting {
                // Hunt just ended
                // info!("Hunt ended. Sage activated during this hunt: {}", *sage_was_activated_during_this_hunt);
                let mut player_has_unconsumed_sage_now = false;
                let gear_iter = player_gear
                    .left_hand
                    .iter()
                    .chain(player_gear.right_hand.iter())
                    .chain(player_gear.inventory.iter());

                for entity in gear_iter {
                    if let Ok(kind) = q_gear.get(*entity)
                        && *kind == GearKind::SageBundle
                        && let Ok(sage_data) = q_sage.get(*entity)
                        && !sage_data.consumed
                    {
                        player_has_unconsumed_sage_now = true;
                        break;
                    }
                }

                if player_has_unconsumed_sage_now && !*sage_was_activated_during_this_hunt {
                    // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
                    walkie_play.set(
                        WalkieEvent::SageUnusedDefensivelyDuringHunt,
                        time.elapsed_secs_f64(),
                    );
                }
                // Reset tracker for the next hunt
                *tracker = HuntSageUsageTracker::default();
            } else {
                // Still hunting, check if player activates sage
                if !*sage_was_activated_during_this_hunt {
                    // Only check if not already flagged
                    let gear_iter = player_gear
                        .left_hand
                        .iter()
                        .chain(player_gear.right_hand.iter())
                        .chain(player_gear.inventory.iter());

                    for entity in gear_iter {
                        if let Ok(kind) = q_gear.get(*entity)
                            && *kind == GearKind::SageBundle
                            && let Ok(sage_data) = q_sage.get(*entity)
                            && sage_data.is_active
                        {
                            *sage_was_activated_during_this_hunt = true;
                            // info!("Sage activated by player during current hunt.");
                            break;
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, quartz_cracked_feedback);
    app.add_systems(Update, quartz_shattered_feedback);
    app.add_systems(Update, trigger_quartz_unused_in_relevant_situation_system);
    app.add_systems(Update, trigger_sage_unused_in_relevant_situation_system);
    app.add_systems(Update, trigger_sage_activated_ineffectively_system);
    app.init_resource::<HuntSageUsageTracker>()
        .add_systems(Update, reset_hunt_sage_tracker_on_mission_change)
        .add_systems(
            Update,
            trigger_sage_unused_defensively_during_hunt_system
                .after(reset_hunt_sage_tracker_on_mission_change),
        );
}
