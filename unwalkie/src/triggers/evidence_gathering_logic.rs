use bevy::prelude::*;
use uncore::components::player_sprite::PlayerSprite;
use uncore::resources::board_data::BoardData;
use uncore::resources::current_evidence_readings::CurrentEvidenceReadings;
use uncore::resources::ghost_guess::GhostGuess;
use uncore::states::AppState;
use uncore::states::GameState;
use uncore::types::evidence::Evidence;
use uncore::types::gear_kind::GearKind;
use ungear::components::playergear::PlayerGear;
use untruck::uibutton::TruckButtonState;
use untruck::uibutton::TruckButtonType;
use untruck::uibutton::TruckUIButton;
use unwalkiecore::WalkieEvent;
use unwalkiecore::WalkiePlay;

fn trigger_journal_points_to_one_ghost_no_craft_on_exit_v3_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut prev_game_state: Local<GameState>,
    mut walkie_play: ResMut<WalkiePlay>,
    gg: Res<GhostGuess>, // GhostGuess resource is the key
    player_query: Query<&PlayerGear, With<PlayerSprite>>,
    // We don't strictly need truck_tab_query or truck_button_query if gg is reliable
) {
    if *app_state.get() != AppState::InGame {
        *prev_game_state = *game_state.get();
        return;
    }

    let current_gs_val = *game_state.get();
    let previous_gs_val = *prev_game_state;
    *prev_game_state = current_gs_val;

    // --- Check conditions when player HAS JUST LEFT THE TRUCK ---
    if current_gs_val == GameState::None && previous_gs_val == GameState::Truck {
        if let Some(_identified_ghost_via_guess) = gg.ghost_type {
            // Player just left the truck, and GhostGuess indicates a single ghost was selected.

            // Check if player has the correct repellent for this guessed ghost.
            let Ok(player_gear) = player_query.get_single() else {
                return;
            };
            let mut player_has_repellent = false;
            for (gear, _epos) in player_gear.as_vec() {
                if gear.kind == GearKind::RepellentFlask {
                    player_has_repellent = true;
                }
            }

            if !player_has_repellent {
                // They left the truck after their journal pointed to one ghost (via GhostGuess),
                // and they don't have the correct repellent in their inventory.
                // Add a small delay here using a Local timer if desired, or rely on WalkiePlay cooldowns.
                // For now, direct trigger:
                walkie_play.set(
                    WalkieEvent::JournalPointsToOneGhostNoCraft,
                    time.elapsed_secs_f64(),
                );
            }
        }
        // If gg.ghost_type is None, it means either no single ghost was ID'd or player cleared their guess.
        // In this case, the condition for the hint isn't met.
    }
}

const DELAY_AFTER_INCORRECT_MARKING_SECONDS: f32 = 10.0;

#[derive(PartialEq, Clone, Debug, Default)]
struct IncorrectEvidenceMarkedState {
    // We only care about EMF5 for this specific event.
    // If we generalize, this could hold Option<Evidence>.
    emf5_incorrectly_marked_since: Option<f32>, // Timestamp when EMF5 was first detected as incorrectly marked
}

fn trigger_emf_non_emf5_fixation_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    truck_button_query: Query<&TruckUIButton>,
    board_data: Res<BoardData>,
    mut incorrect_marker_state: Local<IncorrectEvidenceMarkedState>,
    // TODO: player_profile: Res<Persistent<PlayerProfileData>>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        // If not in the right state, reset the timer state
        *incorrect_marker_state = IncorrectEvidenceMarkedState::default();
        return;
    }

    // 2. Check Journal State for EMF5
    let mut emf5_button_is_pressed_in_journal = false;
    for button_data in truck_button_query.iter() {
        if button_data.class == TruckButtonType::Evidence(Evidence::EMFLevel5) {
            if button_data.status == TruckButtonState::Pressed {
                emf5_button_is_pressed_in_journal = true;
            }
            break; // Found the EMF5 button
        }
    }

    // 3. Check Actual Ghost Evidences
    let ghost_actually_has_emf5 = board_data.evidences.contains(&Evidence::EMFLevel5);

    // 4. Detect Conflict
    let conflict_exists = emf5_button_is_pressed_in_journal && !ghost_actually_has_emf5;

    // 5. Manage Timer and Trigger
    if conflict_exists {
        if incorrect_marker_state
            .emf5_incorrectly_marked_since
            .is_none()
        {
            // Conflict just started (or was re-detected after a reset)
            incorrect_marker_state.emf5_incorrectly_marked_since = Some(time.elapsed_secs());
        } else if let Some(start_time) = incorrect_marker_state.emf5_incorrectly_marked_since {
            let duration_of_conflict = time.elapsed_secs() - start_time;
            if duration_of_conflict > DELAY_AFTER_INCORRECT_MARKING_SECONDS {
                // TODO: Add PlayerProfileData check here to limit hint frequency for experienced players.
                // For example:
                // if player_profile.statistics.total_missions_completed < 5 ||
                //    !player_profile.achievements.some_emf_understanding_achieved { ... }

                if walkie_play.set(WalkieEvent::EMFNonEMF5Fixation, time.elapsed_secs_f64()) {
                    // Hint was played, reset the state to prevent immediate re-trigger
                    *incorrect_marker_state = IncorrectEvidenceMarkedState::default();
                }
            }
        }
    } else {
        // No conflict, or conflict resolved
        if incorrect_marker_state
            .emf5_incorrectly_marked_since
            .is_some()
        {
            *incorrect_marker_state = IncorrectEvidenceMarkedState::default();
        }
    }
}

const CONFLICT_DURATION_THRESHOLD_SECONDS: f32 = 60.0;

// Local state to track how long a conflicting evidence state has persisted
#[derive(Default)]
struct ConflictingEvidenceTracker {
    conflict_active_since_timestamp: Option<f32>,
}

fn trigger_journal_conflicting_evidence_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    truck_button_query: Query<&TruckUIButton>,
    board_data: Res<BoardData>,
    mut tracker: Local<ConflictingEvidenceTracker>,
    // TODO: player_profile: Res<Persistent<PlayerProfileData>>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        // If not in the right state, reset the tracker
        *tracker = ConflictingEvidenceTracker::default();
        return;
    }

    // 2. Check for Any Conflicting Evidence
    let mut current_conflict_exists = false;
    for button_data in truck_button_query.iter() {
        if let TruckButtonType::Evidence(marked_evidence_type) = button_data.class {
            let actual_ghost_has_this_evidence =
                board_data.evidences.contains(&marked_evidence_type);

            if button_data.status == TruckButtonState::Pressed && !actual_ghost_has_this_evidence {
                current_conflict_exists = true;
                break;
            }
            if button_data.status == TruckButtonState::Discard && actual_ghost_has_this_evidence {
                current_conflict_exists = true;
                break;
            }
        }
    }

    // 3. Manage Timer and Trigger
    if current_conflict_exists {
        if tracker.conflict_active_since_timestamp.is_none() {
            // A conflict just started (or was re-detected after a reset/previous clear state)
            tracker.conflict_active_since_timestamp = Some(time.elapsed_secs());
        } else if let Some(start_time) = tracker.conflict_active_since_timestamp {
            let duration_of_conflict = time.elapsed_secs() - start_time;
            if duration_of_conflict > CONFLICT_DURATION_THRESHOLD_SECONDS {
                // TODO: Add PlayerProfileData check here.

                if walkie_play.set(
                    WalkieEvent::JournalConflictingEvidence,
                    time.elapsed_secs_f64(),
                ) {
                    // Hint was played, reset the timer to prevent immediate re-trigger for this same conflict.
                    // The conflict might still exist, but we've hinted.
                    // It will re-arm if the conflict resolves and then a *new* one appears.
                    tracker.conflict_active_since_timestamp = None;
                }
            }
        }
    } else {
        // No conflict currently exists
        if tracker.conflict_active_since_timestamp.is_some() {
            // Conflict was just resolved
            tracker.conflict_active_since_timestamp = None;
        }
    }
}

// How clear evidence needs to be to trigger the "confirmed" hint.
const CLEAR_EVIDENCE_CONFIRMATION_THRESHOLD: f32 = 0.5;

/// Generic system to trigger "Evidence Confirmed" walkie events.
fn trigger_evidence_confirmed_feedback_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    evidence_readings: Res<CurrentEvidenceReadings>,
    truck_button_query: Query<&TruckUIButton>,
    // TODO: player_profile: Res<Persistent<PlayerProfileData>>, // For experience-based limiting
) {
    // System Run Condition
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        return;
    }
    use enum_iterator::all;

    for evidence_type in all::<Evidence>() {
        // Iterate through all defined Evidence types
        if let Some(reading) = evidence_readings.get_reading(evidence_type) {
            if reading.clarity >= CLEAR_EVIDENCE_CONFIRMATION_THRESHOLD {
                // Evidence is currently clearly visible/audible

                // Check if player has already marked this evidence in their journal
                let mut player_already_marked_evidence = false;
                for button_data in truck_button_query.iter() {
                    if button_data.class == TruckButtonType::Evidence(evidence_type) {
                        if button_data.status == TruckButtonState::Pressed {
                            player_already_marked_evidence = true;
                        }
                        break; // Found the button for this evidence type
                    }
                }

                // Skip hint if player has already acknowledged this evidence
                if player_already_marked_evidence {
                    continue;
                }

                // TODO: Add PlayerProfileData check here to limit hints for experienced players
                // e.g., if player_profile.level > 5 && evidence_type == Evidence::FreezingTemp { continue; }

                let walkie_event_to_send = match evidence_type {
                    Evidence::FreezingTemp => Some(WalkieEvent::FreezingTempsEvidenceConfirmed),
                    Evidence::FloatingOrbs => Some(WalkieEvent::FloatingOrbsEvidenceConfirmed),
                    Evidence::UVEctoplasm => Some(WalkieEvent::UVEctoplasmEvidenceConfirmed),
                    Evidence::EMFLevel5 => Some(WalkieEvent::EMFLevel5EvidenceConfirmed),
                    Evidence::EVPRecording => Some(WalkieEvent::EVPEvidenceConfirmed),
                    Evidence::SpiritBox => Some(WalkieEvent::SpiritBoxEvidenceConfirmed),
                    Evidence::RLPresence => Some(WalkieEvent::RLPresenceEvidenceConfirmed),
                    Evidence::CPM500 => Some(WalkieEvent::CPM500EvidenceConfirmed),
                };

                if let Some(event_to_send) = walkie_event_to_send {
                    // Attempt to set the event. If successful, mark it in the tracker.
                    if walkie_play.set(event_to_send, time.elapsed_secs_f64()) {
                        // info!("[Walkie] Triggered {:?} confirmation.", evidence_type);
                        walkie_play.set_evidence_hint(evidence_type, time.elapsed_secs_f64());
                    }
                }
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        trigger_journal_points_to_one_ghost_no_craft_on_exit_v3_system,
    );
    app.add_systems(Update, trigger_emf_non_emf5_fixation_system);
    app.add_systems(Update, trigger_journal_conflicting_evidence_system);
    app.add_systems(Update, trigger_evidence_confirmed_feedback_system);
}
