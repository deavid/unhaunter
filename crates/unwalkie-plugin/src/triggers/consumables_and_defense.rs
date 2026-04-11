use bevy::prelude::*;
use unboard_core::resources::roomdb::RoomTopology;
use uncommon_states_core::UIContextState;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::types::gear::kind::GearKind;
use ungearitems_core::components::quartz::QuartzStoneData;
use ungearitems_core::components::sage::SageBundleData;
use unghost_core::components::logic::ghost_sprite::GhostSprite;
use unghost_core::resources::signals::GhostHuntSignals;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unspatial_core::position::Position;
use untruck_core::truckgear::TruckGear;
use unwalkie_core::messages::ProposeWalkieEvent;
use unwalkie_core::resources::WalkiePlay;

/// Triggers a feedback event when the player's quartz stone cracks, after the hunt is over or player leaves the location.
fn quartz_cracked_feedback(
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    qp: Query<(&PlayerSprite, &Position, &PlayerGear)>,
    q_quartz: Query<&QuartzStoneData>,
    room_topology: Res<RoomTopology>,
    app_state: Res<State<UIContextState>>,
    time: Res<Time>,
    mut last_cracks: Local<Option<u8>>,
) {
    if app_state.get() != &UIContextState::InGame {
        *last_cracks = None;
        return;
    }
    let Some((_player, pos, gear)) = qp.iter().next() else {
        return;
    };
    let player_bpos = pos.to_board_position();
    if room_topology.room_tiles.get(&player_bpos).is_none() {
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
                crate::triggers::net::walkie_set_or_propose(
                    unwalkie_core::events::walkie_types::WalkieEvent::QuartzCrackedFeedback,
                    time.elapsed_secs_f64(),
                    &mut walkie_play,
                    &mut ev_propose,
                );
            }
            *last_cracks = Some(quartz.cracks);
        }
    }
}

/// Triggers a feedback event when the player's quartz stone shatters, after the hunt is over or player leaves the location.
fn quartz_shattered_feedback(
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    qp: Query<(&PlayerSprite, &Position, &PlayerGear)>,
    q_quartz: Query<&QuartzStoneData>,
    room_topology: Res<RoomTopology>,
    app_state: Res<State<UIContextState>>,
    time: Res<Time>,
    mut shattered: Local<bool>,
) {
    if app_state.get() != &UIContextState::InGame {
        *shattered = false;
        return;
    }
    let Some((_player, pos, gear)) = qp.iter().next() else {
        return;
    };
    let player_bpos = pos.to_board_position();
    if room_topology.room_tiles.get(&player_bpos).is_none() {
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
            crate::triggers::net::walkie_set_or_propose(
                unwalkie_core::events::walkie_types::WalkieEvent::QuartzShatteredFeedback,
                time.elapsed_secs_f64(),
                &mut walkie_play,
                &mut ev_propose,
            );
            *shattered = true;
        }
    }
}

fn trigger_quartz_unused_in_relevant_situation_system(
    time: Res<Time>,
    app_state: Res<State<UIContextState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    player_query: Query<(&PlayerGear, &Position), (With<PlayerSprite>, With<MainPlayer>)>,
    hunt_signals: Res<GhostHuntSignals>,
    difficulty: Res<CurrentDifficulty>,
    truck_gear: Option<Res<TruckGear>>,
    room_topology: Res<RoomTopology>,
    q_gear: Query<&GearKind>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != UIContextState::InGame {
        return;
    }

    // 2. Chapter Check: Only trigger for Chapter 5 or non-tutorial difficulties
    let current_chapter_index = difficulty.0.index();
    if current_chapter_index < undifficulty_core::difficulty::Difficulty::TutorialChapter5.index() {
        // If it's a tutorial chapter AND it's before Chapter 5, exit.
        // Non-tutorial difficulties (where tutorial_chapter is None, so current_chapter_index is usize::MAX) will pass this.
        return;
    }

    if !hunt_signals.any_hunted_this_mission || !hunt_signals.any_hunt_likely {
        return;
    }

    for (player_gear, player_pos) in player_query.iter() {
        {
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
                continue; // Player already has quartz, no need for this hint
            }

            // 7. Check Truck Inventory for Quartz
            // Only trigger truck hint if player is currently outside (near truck)
            let player_bpos = player_pos.to_board_position();
            let is_outside = room_topology.room_tiles.get(&player_bpos).is_none();
            if !is_outside {
                continue; // Don't nag about truck gear while inside
            }

            let truck_gear = match truck_gear.as_ref() {
                Some(gear) => gear,
                None => continue, // No truck gear available, exit early
            };
            let truck_has_quartz = truck_gear.inventory.iter().any(|&gear| check_gear(gear));
            if !truck_has_quartz {
                continue; // Quartz isn't even available in the truck
            }

            // 8. Trigger Event: All conditions met
            if crate::triggers::net::walkie_set_or_propose(
                unwalkie_core::events::walkie_types::WalkieEvent::QuartzUnusedInRelevantSituation,
                time.elapsed_secs_f64(),
                &mut walkie_play,
                &mut ev_propose,
            ) {
                return;
            }
        }
    }
}

fn trigger_sage_unused_in_relevant_situation_system(
    time: Res<Time>,
    app_state: Res<State<UIContextState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    player_query: Query<(&PlayerGear, &Position), (With<PlayerSprite>, With<MainPlayer>)>,
    hunt_signals: Res<GhostHuntSignals>,
    difficulty: Res<CurrentDifficulty>,
    truck_gear: Option<Res<TruckGear>>,
    room_topology: Res<RoomTopology>,
    q_gear: Query<&GearKind>,
    q_sage: Query<&SageBundleData>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != UIContextState::InGame {
        return;
    }

    // 2. Chapter Check: Only trigger for Chapter 5 or non-tutorial difficulties
    let current_chapter_index = difficulty.0.index();
    if current_chapter_index < 4 {
        // If it's a tutorial chapter AND it's before Chapter 5 (index 4), exit.
        // Non-tutorial difficulties (index 5+) will pass this.
        return;
    }

    if !hunt_signals.any_hunted_this_mission || !hunt_signals.any_hunt_likely {
        return;
    }

    for (player_gear, player_pos) in player_query.iter() {
        {
            // 6. Check Player Inventory for Sage
            let mut player_has_unlit_sage = false;
            let mut player_has_active_sage = false;

            let mut check_gear_status = |entity: Entity| {
                if let Ok(kind) = q_gear.get(entity)
                    && *kind == GearKind::SageBundle
                    && let Ok(sage_data) = q_sage.get(entity)
                    && !sage_data.consumed
                {
                    if sage_data.is_active {
                        player_has_active_sage = true;
                    } else {
                        player_has_unlit_sage = true;
                    }
                }
            };

            if let Some(e) = player_gear.left_hand {
                check_gear_status(e);
            }
            if let Some(e) = player_gear.right_hand {
                check_gear_status(e);
            }
            for &e in &player_gear.inventory {
                check_gear_status(e);
            }

            if player_has_active_sage {
                continue; // Already protected
            }

            if player_has_unlit_sage {
                // Trigger hint to light it up!
                if crate::triggers::net::walkie_set_or_propose(
                    unwalkie_core::events::walkie_types::WalkieEvent::SageUnusedInRelevantSituation,
                    time.elapsed_secs_f64(),
                    &mut walkie_play,
                    &mut ev_propose,
                ) {
                    return;
                }
                continue;
            }

            // 7. Player has no usable sage in inventory.
            // Check if player is outside (near truck) to suggest picking it up.
            let player_bpos = player_pos.to_board_position();
            let is_outside = room_topology.room_tiles.get(&player_bpos).is_none();
            if !is_outside {
                continue; // Don't nag if they've already committed to being inside without it.
            }

            let truck_gear = match truck_gear.as_ref() {
                Some(gear) => gear,
                None => continue, // No truck gear available, exit early
            };

            let check_gear_kind = |entity: Entity| -> bool {
                if let Ok(kind) = q_gear.get(entity) {
                    *kind == GearKind::SageBundle
                } else {
                    false
                }
            };
            let truck_has_sage = truck_gear
                .inventory
                .iter()
                .any(|&gear| check_gear_kind(gear));
            if !truck_has_sage {
                continue; // Sage isn't even available in the truck
            }

            // 8. Trigger Event: All conditions met
            if crate::triggers::net::walkie_set_or_propose(
                unwalkie_core::events::walkie_types::WalkieEvent::SageUnusedInRelevantSituation,
                time.elapsed_secs_f64(),
                &mut walkie_play,
                &mut ev_propose,
            ) {
                return;
            }
        }
    }
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
    app_state: Res<State<UIContextState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    player_query: Query<(Entity, &PlayerGear), (With<PlayerSprite>, With<MainPlayer>)>, // Added Entity to ID player
    ghost_query: Query<&GhostSprite>,
    difficulty: Res<CurrentDifficulty>,
    mut tracker: Local<SageEffectivenessTracker>,
    q_gear: Query<&GearKind>,
    q_sage: Query<&SageBundleData>,
) {
    // 1. System Run Condition & Chapter Check & Reset conditions
    if *app_state.get() != UIContextState::InGame {
        if tracker.is_tracking_this_sage_burn {
            *tracker = SageEffectivenessTracker::default();
        }
        return;
    }
    let current_chapter_index = difficulty.0.index();
    if current_chapter_index < 4 {
        if tracker.is_tracking_this_sage_burn {
            *tracker = SageEffectivenessTracker::default();
        }
        return;
    }

    // 2. Get Player & Ghost Info
    let mut _any_player_handled = false;
    for (player_entity, player_gear) in player_query.iter() {
        for ghost_sprite in ghost_query.iter() {
            _any_player_handled = true;
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
                if tracker.is_tracking_this_sage_burn
                    && tracker.player_entity_id == Some(player_entity)
                {
                    *tracker = SageEffectivenessTracker::default();
                    return;
                }
                continue;
            };

            // 4. Manage Tracker State
            if sage_data.is_active && !sage_data.consumed {
                // Sage is currently burning
                if !tracker.is_tracking_this_sage_burn
                    || tracker.player_entity_id != Some(player_entity)
                {
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
                if tracker.is_tracking_this_sage_burn
                    && tracker.player_entity_id == Some(player_entity)
                {
                    // Sage was being tracked for this player, and now it's no longer active.
                    // This means it was either consumed or deactivated (e.g. player dropped/stowed it).
                    // If it was consumed, this is when we check effectiveness.
                    if sage_data.consumed {
                        // Check the consumed flag
                        let calm_increase =
                            ghost_sprite.calm_time_secs - tracker.initial_ghost_calm_time_secs;
                        if calm_increase < MIN_EFFECTIVE_SAGE_CALM_INCREASE {
                            // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
                            crate::triggers::net::walkie_set_or_propose(
                                unwalkie_core::events::walkie_types::WalkieEvent::SageActivatedIneffectively,
                                time.elapsed_secs_f64(),
                                &mut walkie_play,
                                &mut ev_propose,
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
                && time.elapsed_secs() - tracker.sage_activated_game_time
                    > SAGE_TRACKING_TIMEOUT_SECONDS
            {
                // info!("Sage tracking timed out for player {:?}. Resetting.", tracker.player_entity_id);
                *tracker = SageEffectivenessTracker::default();
            }
        }
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
    app_state: Res<State<UIContextState>>,
    mut last_app_state: Local<Option<UIContextState>>, // Track previous app state
) {
    let current_app_state = *app_state.get();
    if *last_app_state != Some(current_app_state) {
        // If app state changed or it's the first run
        if current_app_state != UIContextState::InGame
            || (last_app_state.is_some()
                && last_app_state.unwrap() != UIContextState::InGame
                && current_app_state == UIContextState::InGame)
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
    app_state: Res<State<UIContextState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    player_query: Query<&PlayerGear, (With<PlayerSprite>, With<MainPlayer>)>,
    hunt_signals: Res<GhostHuntSignals>,
    difficulty: Res<CurrentDifficulty>,
    mut tracker: ResMut<HuntSageUsageTracker>, // Use ResMut for the tracker
    q_gear: Query<&GearKind>,
    q_sage: Query<&SageBundleData>,
) {
    // 1. System Run Condition & Chapter Check
    if *app_state.get() != UIContextState::InGame {
        // Tracker reset is handled by `reset_hunt_sage_tracker_on_mission_change`
        return;
    }
    let current_chapter_index = difficulty.0.index();
    if current_chapter_index < 4 {
        return;
    }

    // 2. Get Player & Ghost Info
    for player_gear in player_query.iter() {
        let current_ghost_is_hunting = hunt_signals.any_hunting;

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
                        crate::triggers::net::walkie_set_or_propose(
                            unwalkie_core::events::walkie_types::WalkieEvent::SageUnusedDefensivelyDuringHunt,
                            time.elapsed_secs_f64(),
                            &mut walkie_play,
                            &mut ev_propose,
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
