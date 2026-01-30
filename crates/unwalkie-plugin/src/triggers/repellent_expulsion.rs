use bevy::prelude::*;
use bevy_platform::collections::HashSet;
use unbehavior::roomdb::RoomDB;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::types::GearKind;
use ungearitems_core::components::repellentflask::RepellentFlask;
use unghost_core::components::ghost_sprite::GhostSprite;
use unghost_core::components::repellent_particle::RepellentParticle;
use unghost_core::types::ghost::types::GhostType;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unspatial_core::position::Position;
use untypes_core::states::{AppState, GameState};
use unwalkie_core::events::WalkieEvent;
use unwalkie_core::resources::WalkiePlay;

/// How long player must linger after ghost is gone
const LINGER_THRESHOLD_SECONDS: f64 = 10.0;

fn trigger_ghost_expelled_player_lingers_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    ghost_query: Query<Entity, With<GhostSprite>>,
    player_query: Query<&Position, (With<PlayerSprite>, With<MainPlayer>)>, // Assuming only one player for now
    roomdb: Res<RoomDB>,
    mut ghost_gone_and_player_in_location_timestamp: Local<Option<f64>>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        // If not in the right state, reset the timer and do nothing
        if ghost_gone_and_player_in_location_timestamp.is_some() {
            *ghost_gone_and_player_in_location_timestamp = None;
        }
        return;
    }

    // 2. Check Ghost Presence
    let ghost_is_present = !ghost_query.is_empty();

    // 3. Check Player Location - iterate all players (First Responder)
    for player_pos in player_query.iter() {
        let player_is_inside_location = roomdb
            .room_tiles
            .get(&player_pos.to_board_position())
            .is_some();

        // 4. Manage Timer and Trigger Logic
        if !ghost_is_present && player_is_inside_location {
            // Ghost is gone AND player is inside the location
            if ghost_gone_and_player_in_location_timestamp.is_none() {
                // Start the timer
                *ghost_gone_and_player_in_location_timestamp = Some(time.elapsed_secs_f64());
            } else if let Some(start_time) = *ghost_gone_and_player_in_location_timestamp {
                let duration_lingering = time.elapsed_secs_f64() - start_time;
                if duration_lingering > LINGER_THRESHOLD_SECONDS
                    && walkie_play.set(
                        WalkieEvent::GhostExpelledPlayerLingers,
                        time.elapsed_secs_f64(),
                    )
                {
                    // Event successfully set, reset timer to prevent immediate re-trigger
                    // for this "lingering session". The global WalkiePlay cooldown will handle mission-level frequency.
                    *ghost_gone_and_player_in_location_timestamp = None;
                }
            }
        } else {
            // Ghost is present OR player is outside, reset the timer
            if ghost_gone_and_player_in_location_timestamp.is_some() {
                *ghost_gone_and_player_in_location_timestamp = None;
            }
        }
    }
}

fn trigger_has_repellent_enters_location_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&PlayerGear, &Position), (With<PlayerSprite>, With<MainPlayer>)>,
    roomdb: Res<RoomDB>,
    q_gear: Query<&GearKind>,
    q_repellent: Query<&RepellentFlask>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        return;
    }

    // Iterate all players (First Responder)
    for (player_gear, player_pos) in player_query.iter() {
        // 3. Check Repellent Status
        let check_repellent = |entity: Entity| -> bool {
            if let Ok(kind) = q_gear.get(entity)
                && *kind == GearKind::RepellentFlask
                && let Ok(repellent) = q_repellent.get(entity)
            {
                return repellent.qty > 0;
            }
            false
        };

        let has_valid_repellent = player_gear.left_hand.map(check_repellent).unwrap_or(false)
            || player_gear.right_hand.map(check_repellent).unwrap_or(false)
            || player_gear.inventory.iter().any(|&e| check_repellent(e));

        // 4. Determine Current Location Status
        let player_is_currently_inside = roomdb
            .room_tiles
            .get(&player_pos.to_board_position())
            .is_some();

        if player_is_currently_inside && has_valid_repellent {
            walkie_play.set(
                WalkieEvent::HasRepellentEntersLocation,
                time.elapsed_secs_f64(),
            );
            return; // First responder wins
        }
    }
}

const EFFECTIVE_REPELLENT_RANGE: f32 = 3.0;
const TOO_FAR_DURATION_SECONDS: f64 = 5.0;

// Local state to track when the "too far" condition started
#[derive(Default)]
struct PrevRepellentState {
    was_active: bool,
    too_far_started: Option<f64>,
}

fn trigger_repellent_used_too_far_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&PlayerGear, &Position), (With<PlayerSprite>, With<MainPlayer>)>,
    ghost_query: Query<(&Position, &GhostSprite), Without<PlayerSprite>>,
    mut prev_repellent_state: Local<PrevRepellentState>,
    q_gear: Query<&GearKind>,
    q_repellent: Query<&RepellentFlask>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        prev_repellent_state.was_active = false; // Reset on state change
        return;
    }

    // Iterate all players and ghosts (Simulation pattern - multi-entity)
    for (player_gear, player_pos) in player_query.iter() {
        for (ghost_pos, ghost_sprite) in ghost_query.iter() {
            // 2. Check current repellent state
            let mut current_repellent_is_active = false;
            let check_repellent = |entity: Entity| -> bool {
                if let Ok(kind) = q_gear.get(entity)
                    && *kind == GearKind::RepellentFlask
                    && let Ok(repellent) = q_repellent.get(entity)
                {
                    return repellent.active && repellent.qty > 0;
                }
                false
            };

            if player_gear.left_hand.map(check_repellent).unwrap_or(false)
                || player_gear.right_hand.map(check_repellent).unwrap_or(false)
                || player_gear.inventory.iter().any(|&e| check_repellent(e))
            {
                current_repellent_is_active = true;
            }

            // 3. Check if repellent is active and player is too far
            if current_repellent_is_active {
                let target_pos: Position = *ghost_pos;

                if ghost_sprite.get_health() < 0.5 {
                    // Don't warn on this if the ghost is about to die.
                    continue;
                }
                let distance = player_pos.distance(&target_pos);
                let is_too_far = distance > EFFECTIVE_REPELLENT_RANGE;

                if is_too_far {
                    if prev_repellent_state.too_far_started.is_none() {
                        prev_repellent_state.too_far_started = Some(time.elapsed_secs_f64());
                    } else if let Some(start_time) = prev_repellent_state.too_far_started
                        && time.elapsed_secs_f64() - start_time >= TOO_FAR_DURATION_SECONDS
                    {
                        walkie_play.set(WalkieEvent::RepellentUsedTooFar, time.elapsed_secs_f64());
                        prev_repellent_state.too_far_started = None; // Reset after triggering
                    }
                } else {
                    prev_repellent_state.too_far_started = None; // Reset if not too far
                }
            } else {
                prev_repellent_state.too_far_started = None; // Reset if repellent not active
            }
        }
    }
}

const REACTION_WINDOW_SECONDS: f32 = 5.0;
const PARTICLE_NEARBY_THRESHOLD: f32 = 3.5; // How close particles need to be to the ghost

#[derive(Default)]
struct RepellentReactionTracker {
    repellent_activated_time: f32,
    initial_ghost_hunting_state: f32, // Using f32 to directly compare with GhostSprite.hunting
                                      // Potentially add ghost_entity_id if multiple ghosts were possible
}

// Local state to track if the repellent was active in the previous frame for activation detection
#[derive(Default)]
struct PrevRepellentActiveState {
    was_active: bool,
}

fn trigger_repellent_provokes_strong_reaction_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<(&PlayerGear, &Position), (With<PlayerSprite>, With<MainPlayer>)>,
    mut ghost_query: Query<(&GhostSprite, &Position)>,
    repellent_particle_query: Query<&Position, With<RepellentParticle>>,
    mut tracker: Local<Option<RepellentReactionTracker>>,
    mut prev_rep_active_state: Local<PrevRepellentActiveState>,
    current_difficulty_res: Res<CurrentDifficulty>,
    q_gear: Query<&GearKind>,
    q_repellent: Query<&RepellentFlask>,
) {
    let difficulty_info = &current_difficulty_res.0;
    if !difficulty_info.difficulty.is_tutorial_difficulty() {
        return;
    }

    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        *tracker = None;
        prev_rep_active_state.was_active = false;
        return;
    }

    // Iterate all players and ghosts (Simulation pattern)
    for (player_gear, _player_pos) in player_query.iter() {
        for (ghost_sprite, ghost_pos) in ghost_query.iter_mut() {
            // 2. Detect Player Repellent Activation
            let mut current_repellent_is_active_and_has_qty = false;
            let check_repellent = |entity: Entity| -> bool {
                if let Ok(kind) = q_gear.get(entity)
                    && *kind == GearKind::RepellentFlask
                    && let Ok(repellent) = q_repellent.get(entity)
                {
                    return repellent.active && repellent.qty > 0;
                }
                false
            };

            if player_gear.left_hand.map(check_repellent).unwrap_or(false)
                || player_gear.right_hand.map(check_repellent).unwrap_or(false)
                || player_gear.inventory.iter().any(|&e| check_repellent(e))
            {
                current_repellent_is_active_and_has_qty = true;
            }

            if current_repellent_is_active_and_has_qty && !prev_rep_active_state.was_active {
                // Repellent was just activated this frame by the player
                *tracker = Some(RepellentReactionTracker {
                    repellent_activated_time: time.elapsed_secs(),
                    initial_ghost_hunting_state: ghost_sprite.hunting,
                });
            }
            prev_rep_active_state.was_active = current_repellent_is_active_and_has_qty;

            // 3. Monitor Ghost Reaction (if tracker is active)
            if let Some(tracker_data) = tracker.as_ref() {
                let time_since_activation =
                    time.elapsed_secs() - tracker_data.repellent_activated_time;

                if time_since_activation <= REACTION_WINDOW_SECONDS {
                    let hunt_just_started = ghost_sprite.hunting > 0.0
                        && tracker_data.initial_ghost_hunting_state == 0.0;
                    // Also consider if hunt_warning_active just became true, if initial_ghost_hunting_state was low and warning was false
                    let warning_just_started = ghost_sprite.hunt_warning_active
                        && ghost_sprite.hunting < 1.0
                        && tracker_data.initial_ghost_hunting_state < 1.0;

                    let particles_nearby = repellent_particle_query.iter().any(|particle_pos| {
                        ghost_pos.distance(particle_pos) < PARTICLE_NEARBY_THRESHOLD
                    });
                    if (hunt_just_started || warning_just_started)
                        && particles_nearby
                        && walkie_play.set(
                            WalkieEvent::RepellentUsedGhostEnragesPlayerFlees,
                            time.elapsed_secs_f64(),
                        )
                    {
                        *tracker = None; // Reset tracker after successful trigger
                    }
                } else {
                    // Window has passed
                    *tracker = None;
                }
            }
        }
    }
}

// The concept of `just_emptied_with_type` is now handled by RepellentFlaskData
// retaining its `liquid_content` (GhostType) even when `qty` becomes 0.
// The `update` method in `RepellentFlaskData` ensures `active` is false and `qty` is 0,
// while `liquid_content` preserves the type of ghost it was filled with.

#[derive(Default)]
struct RepellentExhaustedCheckState {
    // Stores the type of ghost the (now empty) repellent was for, if conditions were met
    pending_check_for_ghost_type: Option<GhostType>,
    // Time when the repellent was confirmed exhausted and correct
    time_exhaustion_confirmed: f32,
}

const MAX_PARTICLE_CLEAR_WAIT_SECONDS: f32 = 10.0; // Max time to wait for particles to clear

fn trigger_repellent_exhausted_correct_type_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_query: Query<&PlayerGear, (With<PlayerSprite>, With<MainPlayer>)>,
    ghost_query: Query<&GhostSprite>,
    repellent_particle_query: Query<Entity, With<RepellentParticle>>,
    mut check_state: Local<RepellentExhaustedCheckState>,
    current_difficulty_res: Res<CurrentDifficulty>,
    q_gear: Query<&GearKind>,
    q_repellent: Query<&RepellentFlask>,
) {
    let difficulty_info = &current_difficulty_res.0;
    if !difficulty_info.difficulty.is_tutorial_difficulty() {
        return;
    }

    // 1. System Run Condition Checks & Reset
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        *check_state = RepellentExhaustedCheckState::default(); // Reset on state change
        return;
    }

    // Iterate all players and ghosts (Simulation pattern)
    for player_gear in player_query.iter() {
        for ghost_sprite in ghost_query.iter() {
            if ghost_sprite.get_health() < 0.0 {
                continue;
            }

            // 2. Detect if a Repellent Flask was emptied and it was of the correct type for the current ghost
            if check_state.pending_check_for_ghost_type.is_none() {
                // Only check for new exhaustion events
                let gear_iter = player_gear
                    .left_hand
                    .iter()
                    .chain(player_gear.right_hand.iter())
                    .chain(player_gear.inventory.iter());

                for entity in gear_iter {
                    if let Ok(kind) = q_gear.get(*entity)
                        && *kind == GearKind::RepellentFlask
                        && let Ok(rep_data) = q_repellent.get(*entity)
                    {
                        // Condition 1: Flask is now empty
                        if rep_data.qty == 0 {
                            // Condition 2: Flask *was* filled with a type (which is still stored in liquid_content)
                            if let Some(flask_content_type) = rep_data.liquid_content {
                                // Condition 3: The flask's content type matches the current ghost's type
                                // Condition 4: The ghost has registered hits from the correct repellent type
                                // (ghost_sprite.repellent_hits implies hits from its own class type)
                                if flask_content_type == ghost_sprite.class
                                    && ghost_sprite.repellent_hits > 0
                                {
                                    // This flask, of the correct type, is now empty, and the ghost was affected.
                                    check_state.pending_check_for_ghost_type =
                                        Some(ghost_sprite.class);
                                    check_state.time_exhaustion_confirmed = time.elapsed_secs();
                                    // `liquid_content` is intentionally not cleared in RepellentFlaskData as per new design.
                                    break; // Found a relevant exhausted flask
                                }
                            }
                        }
                    }
                }
            }

            // 3. If pending check, monitor particle dissipation
            if let Some(confirmed_ghost_type) = check_state.pending_check_for_ghost_type {
                // Ensure ghost is still present and of the same type (should be, but good check)
                if ghost_sprite.class != confirmed_ghost_type {
                    *check_state = RepellentExhaustedCheckState::default(); // Ghost changed type? Unlikely but reset.
                    continue;
                }
                let particles_are_few = repellent_particle_query.iter().count() < 10; // Threshold for "few" particles
                let time_since_exhaustion =
                    time.elapsed_secs() - check_state.time_exhaustion_confirmed;
                // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
                if particles_are_few || time_since_exhaustion > MAX_PARTICLE_CLEAR_WAIT_SECONDS {
                    walkie_play.set(
                        WalkieEvent::RepellentExhaustedGhostPresentCorrectType,
                        time.elapsed_secs_f64(),
                    );
                    *check_state = RepellentExhaustedCheckState::default(); // Reset after triggering
                }
            }
        }
    }
}

// Local resource to track ghost entities for which this hint has already been triggered
// in the current "expulsion event" to avoid multiple triggers if, for some reason,
// a ghost removal is processed across multiple system runs or frames without an intervening
// state change that would clear this.
#[derive(Resource, Default)]
struct ProcessedMissedExpulsionGhosts(HashSet<Entity>);

// System to clear the ProcessedMissedExpulsionGhosts on entering a new game state
// or loading, to ensure it's fresh for each mission.
fn reset_processed_missed_expulsion_ghosts_on_new_mission(
    mut processed_ghosts: ResMut<ProcessedMissedExpulsionGhosts>,
    app_state: Res<State<AppState>>, // For detecting transitions away from InGame
    mut last_app_state: Local<Option<AppState>>,
) {
    let current_app_state = *app_state.get();
    if *last_app_state != Some(current_app_state) {
        // If app state changed (e.g., to MainMenu, Summary, or back to Loading/InGame for a new mission)
        // or if it's the first run, clear the set.
        if current_app_state != AppState::InGame
            || last_app_state.is_some_and(|prev| {
                prev != AppState::InGame && current_app_state == AppState::InGame
            })
        {
            // Clear if we are no longer in game, OR if we just entered InGame (new mission)
            if !processed_ghosts.0.is_empty() {
                // info!("Resetting ProcessedMissedExpulsionGhosts due to state change or new mission.");
                processed_ghosts.0.clear();
            }
        }
    }
    *last_app_state = Some(current_app_state);
}

fn trigger_ghost_expelled_player_missed_simplified_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    // GameState isn't strictly needed if we trigger even if player is in truck,
    // as long as they were outside when the ghost was despawned.
    // mut game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut removed_ghost_query: RemovedComponents<GhostSprite>, // Reacts to GhostSprite removal
    player_query: Query<&Position, (With<PlayerSprite>, With<MainPlayer>)>,
    roomdb: Res<RoomDB>,
    mut processed_ghosts: ResMut<ProcessedMissedExpulsionGhosts>,
) {
    // 1. System Run Condition Check (Primarily AppState::InGame)
    if *app_state.get() != AppState::InGame {
        return;
    }

    if removed_ghost_query.is_empty() {
        return; // No ghosts were removed this frame.
    }

    // Iterate all players (First Responder)
    for player_pos in player_query.iter() {
        let player_is_outside_location = roomdb
            .room_tiles
            .get(&player_pos.to_board_position())
            .is_none();

        for removed_ghost_entity in removed_ghost_query.read() {
            // Check if we've already processed this specific ghost entity for this hint
            // in the current "expulsion wave". This is to prevent re-triggering if, for example,
            // the system runs multiple times before a state change that clears `processed_ghosts`.
            if processed_ghosts.0.contains(&removed_ghost_entity) {
                continue;
            }

            if player_is_outside_location {
                // Player was outside when this ghost entity was despawned.
                // info!(
                //     "Ghost {:?} despawned. Player was outside. Triggering GhostExpelledPlayerMissed.",
                //     removed_ghost_entity
                // );
                walkie_play.set(
                    WalkieEvent::GhostExpelledPlayerMissed,
                    time.elapsed_secs_f64(),
                );
                processed_ghosts.0.insert(removed_ghost_entity); // Mark as processed
            // Since WalkiePlay.set() handles cooldowns, one trigger per despawned ghost is fine.
            // If multiple ghosts are expelled simultaneously, this could lead to multiple hints if player is outside.
            // The global cooldown of the event itself should prevent spam.
            } else {
                // Player was inside, mark as processed so we don't re-check if they step out immediately.
                // info!(
                //    "Ghost {:?} despawned. Player was inside. Not triggering GhostExpelledPlayerMissed.",
                //    removed_ghost_entity
                // );
                processed_ghosts.0.insert(removed_ghost_entity);
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, trigger_ghost_expelled_player_lingers_system);
    app.add_systems(Update, trigger_has_repellent_enters_location_system);
    app.add_systems(Update, trigger_repellent_provokes_strong_reaction_system);
    app.add_systems(Update, trigger_repellent_used_too_far_system);
    app.add_systems(Update, trigger_repellent_exhausted_correct_type_system);
    app.init_resource::<ProcessedMissedExpulsionGhosts>()
        .add_systems(
            Update,
            reset_processed_missed_expulsion_ghosts_on_new_mission,
        )
        .add_systems(
            Update,
            trigger_ghost_expelled_player_missed_simplified_system
                .after(reset_processed_missed_expulsion_ghosts_on_new_mission),
        );
}
