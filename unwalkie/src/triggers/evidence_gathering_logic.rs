use bevy::prelude::*;
use bevy::utils::HashMap;
use enum_iterator::all;
use uncore::{
    components::player_sprite::PlayerSprite,
    resources::{board_data::BoardData, current_evidence_readings::CurrentEvidenceReadings},
    states::{AppState, GameState},
    types::evidence::Evidence, // For identifying gear types
};
use ungear::components::playergear::PlayerGear;
use untruck::uibutton::{TruckButtonState, TruckButtonType, TruckUIButton};
use unwalkiecore::{WalkieEvent, WalkiePlay};

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
            if duration_of_conflict > DELAY_AFTER_INCORRECT_MARKING_SECONDS
                && walkie_play.set(WalkieEvent::EMFNonEMF5Fixation, time.elapsed_secs_f64())
            {
                // Hint was played, reset the state to prevent immediate re-trigger
                *incorrect_marker_state = IncorrectEvidenceMarkedState::default();
            }
        }
    } else if incorrect_marker_state
        .emf5_incorrectly_marked_since
        .is_some()
    {
        *incorrect_marker_state = IncorrectEvidenceMarkedState::default();
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
                // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
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

#[derive(Resource, Default)] // Added derive Resource
struct ClearEvidenceTrackedState {
    tracked_clear_evidence: HashMap<Evidence, f64>, // time when it became clear
}

// How clear evidence needs to be to trigger the "confirmed" hint.
const CLEAR_EVIDENCE_THRESHOLD_FOR_HINT: f32 = 0.5; // Renamed for clarity
const TIME_VISIBLE_FOR_CKEY_HINT_SECONDS: f64 = 10.0;

fn trigger_clear_evidence_no_action_ckey_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    evidence_readings: Res<CurrentEvidenceReadings>,
    player_query: Query<(&PlayerSprite, &PlayerGear)>,
    mut tracked_state: ResMut<ClearEvidenceTrackedState>,
) {
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        tracked_state.tracked_clear_evidence.clear();
        return;
    }

    let Ok((_player_sprite, player_gear)) = player_query.get_single() else {
        tracked_state.tracked_clear_evidence.clear();
        return;
    };

    let current_time = time.elapsed_secs_f64();
    let mut to_remove: Vec<Evidence> = Vec::new();

    for evidence_type in all::<Evidence>() {
        let mut is_evidence_clear_on_active_gear = false;
        if player_gear
            .right_hand
            .kind
            .is_evidence_tool_for(evidence_type)
        {
            if let Some(reading) = evidence_readings.get_reading(evidence_type) {
                if reading.clarity >= CLEAR_EVIDENCE_THRESHOLD_FOR_HINT {
                    is_evidence_clear_on_active_gear = true;
                }
            }
        }

        if is_evidence_clear_on_active_gear {
            let entry = tracked_state
                .tracked_clear_evidence
                .entry(evidence_type)
                .or_insert(current_time);
            if current_time - *entry >= TIME_VISIBLE_FOR_CKEY_HINT_SECONDS {
                // Check if player pressed the journal assign key recently for *this* evidence type
                // This requires knowing which evidence is "targeted" by the key press,
                // which might be complex if not directly tied to active gear.
                // For now, assume if *any* assign key was pressed, it might be for this.
                // A more robust check would be needed.
                // The original diff for hint_acknowledge_system.rs was rejected, so we can't rely on that change.
                // Let's assume for now if the hint fires, it's valid.
                // A simple check: did the player recently press the "change evidence" key (C)?
                // This is not ideal as it's not "assign evidence".
                // This hint might be hard to implement correctly without better state tracking of journal interaction.

                // If we assume the player *hasn't* acknowledged it via C_KEY (which is hard to check here without more context
                // on how C_KEY interaction is recorded globally or against specific evidence), we'd fire the hint.
                if walkie_play.set(WalkieEvent::ClearEvidenceFoundNoActionCKey, current_time) {
                    // info!("[Walkie] Triggered ClearEvidenceFoundNoActionCKey for {:?}.", evidence_type);
                    // Mark this specific evidence as hinted to avoid re-triggering immediately
                    // This could be done by removing it or updating its timestamp
                    to_remove.push(evidence_type);
                }
            }
        } else {
            // Evidence is no longer clear for this type, remove from tracking
            if tracked_state
                .tracked_clear_evidence
                .contains_key(&evidence_type)
            {
                to_remove.push(evidence_type);
            }
        }
    }
    for ev_type in to_remove {
        tracked_state.tracked_clear_evidence.remove(&ev_type);
    }
}

#[derive(Resource, Default)] // Added derive Resource
struct NoActionTruckTrackedState {
    tracked_for_no_action_truck: HashMap<Evidence, f64>, // time when it became clear and unlogged
}
const TIME_UNLOGGED_FOR_TRUCK_HINT_SECONDS: f64 = 45.0;

fn trigger_clear_evidence_no_action_truck_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    evidence_readings: Res<CurrentEvidenceReadings>,
    truck_button_query: Query<&TruckUIButton>,
    mut tracked_state: ResMut<NoActionTruckTrackedState>,
) {
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        // Only trigger this hint when player is NOT in the truck
        tracked_state.tracked_for_no_action_truck.clear();
        return;
    }
    let current_time = time.elapsed_secs_f64();
    let mut to_remove: Vec<Evidence> = Vec::new();

    for evidence_type in all::<Evidence>() {
        let mut is_evidence_clear_globally = false;
        if let Some(reading) = evidence_readings.get_reading(evidence_type) {
            if reading.clarity >= CLEAR_EVIDENCE_THRESHOLD_FOR_HINT {
                is_evidence_clear_globally = true;
            }
        }

        let mut player_already_marked_evidence = false;
        for button_data in truck_button_query.iter() {
            if button_data.class == TruckButtonType::Evidence(evidence_type) {
                if button_data.status == TruckButtonState::Pressed {
                    player_already_marked_evidence = true;
                }
                break;
            }
        }

        if is_evidence_clear_globally && !player_already_marked_evidence {
            let entry = tracked_state
                .tracked_for_no_action_truck
                .entry(evidence_type)
                .or_insert(current_time);
            if current_time - *entry >= TIME_UNLOGGED_FOR_TRUCK_HINT_SECONDS
                && walkie_play.set(WalkieEvent::ClearEvidenceFoundNoActionTruck, current_time)
            {
                // info!("[Walkie] Triggered ClearEvidenceFoundNoActionTruck for {:?}.", evidence_type);
                to_remove.push(evidence_type);
            }
        } else {
            // Evidence no longer clear, or player marked it
            if tracked_state
                .tracked_for_no_action_truck
                .contains_key(&evidence_type)
            {
                to_remove.push(evidence_type);
            }
        }
    }
    for ev_type in to_remove {
        tracked_state.tracked_for_no_action_truck.remove(&ev_type);
    }
}

const MIN_EVIDENCE_COUNT_FOR_NO_JOURNAL_HINT: usize = 1;
const TIME_IN_TRUCK_NO_JOURNAL_ACTION_SECONDS: f64 = 20.0;

#[derive(Resource, Default)]
struct InTruckNoJournalActionState {
    time_entered_truck_with_unlogged_evidence: Option<f64>,
    hinted_this_truck_session: bool,
}

fn trigger_in_truck_with_evidence_no_journal_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    evidence_readings: Res<CurrentEvidenceReadings>,
    truck_button_query: Query<&TruckUIButton>,
    mut system_state: ResMut<InTruckNoJournalActionState>,
) {
    if *app_state.get() != AppState::InGame {
        system_state.time_entered_truck_with_unlogged_evidence = None;
        system_state.hinted_this_truck_session = false;
        return;
    }

    let current_time = time.elapsed_secs_f64();

    if *game_state.get() == GameState::Truck {
        if system_state.hinted_this_truck_session {
            return; // Already hinted this session
        }

        let mut unlogged_clear_evidence_count = 0;
        let mut player_interacted_with_journal_this_frame = false;

        for evidence_type in all::<Evidence>() {
            let mut is_evidence_clear = false;
            if let Some(reading) = evidence_readings.get_reading(evidence_type) {
                if reading.clarity >= CLEAR_EVIDENCE_THRESHOLD_FOR_HINT {
                    is_evidence_clear = true;
                }
            }

            if is_evidence_clear {
                let mut player_marked_this_evidence = false;
                for button_data in truck_button_query.iter() {
                    if button_data.class == TruckButtonType::Evidence(evidence_type) {
                        if button_data.status == TruckButtonState::Pressed {
                            player_marked_this_evidence = true;
                        }
                        // A more robust check: did the button state *change* this frame to Pressed?
                        // This requires tracking previous state or listening to UI events.
                        // For simplicity, if any evidence button is pressed now, consider it interaction.
                        if truck_button_query.iter().any(|b| {
                            b.status == TruckButtonState::Pressed
                                && matches!(b.class, TruckButtonType::Evidence(_))
                        }) {
                            player_interacted_with_journal_this_frame = true;
                        }
                        break;
                    }
                }
                if !player_marked_this_evidence {
                    unlogged_clear_evidence_count += 1;
                }
            }
        }

        if unlogged_clear_evidence_count >= MIN_EVIDENCE_COUNT_FOR_NO_JOURNAL_HINT {
            if system_state
                .time_entered_truck_with_unlogged_evidence
                .is_none()
            {
                system_state.time_entered_truck_with_unlogged_evidence = Some(current_time);
            }

            if let Some(time_entered) = system_state.time_entered_truck_with_unlogged_evidence {
                if !player_interacted_with_journal_this_frame
                    && (current_time - time_entered >= TIME_IN_TRUCK_NO_JOURNAL_ACTION_SECONDS)
                {
                    // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
                    if walkie_play.set(WalkieEvent::InTruckWithEvidenceNoJournal, current_time) {
                        // info!("[Walkie] Triggered InTruckWithEvidenceNoJournal.");
                        system_state.hinted_this_truck_session = true;
                        // system_state.time_entered_truck_with_unlogged_evidence = None; // Reset after hinting
                    }
                } else if player_interacted_with_journal_this_frame {
                    // Player interacted, reset timer for this session, don't hint now
                    system_state.time_entered_truck_with_unlogged_evidence = None;
                    // system_state.hinted_this_truck_session = true; // Or mark as hinted because they took action
                }
            }
        } else {
            // Not enough unlogged evidence, or all evidence logged
            system_state.time_entered_truck_with_unlogged_evidence = None;
            // system_state.hinted_this_truck_session = false; // Allow re-hint if new unlogged evidence appears
        }
    } else {
        // Not in GameState::Truck
        system_state.time_entered_truck_with_unlogged_evidence = None;
        system_state.hinted_this_truck_session = false;
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
    app.add_systems(Update, trigger_emf_non_emf5_fixation_system);
    app.add_systems(Update, trigger_journal_conflicting_evidence_system);
    app.init_resource::<ClearEvidenceTrackedState>();
    app.init_resource::<NoActionTruckTrackedState>();
    app.init_resource::<InTruckNoJournalActionState>();
    app.add_systems(Update, trigger_clear_evidence_no_action_ckey_system);
    app.add_systems(Update, trigger_clear_evidence_no_action_truck_system);
    app.add_systems(Update, trigger_in_truck_with_evidence_no_journal_system);
    app.add_systems(Update, trigger_evidence_confirmed_feedback_system);
}
