use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::{
    difficulty::CurrentDifficulty,
    resources::{
        current_evidence_readings::CurrentEvidenceReadings, ghost_guess::GhostGuess,
        potential_id_timer::PotentialIDTimer,
    },
    types::evidence::Evidence,
};
use unprofile::data::PlayerProfileData;
use unwalkiecore::{events::WalkieEvent, resources::WalkiePlay};

// PotentialIDTimer struct definition removed from here

pub fn potential_id_prompt_system(
    mut timer: ResMut<PotentialIDTimer>,
    current_evidence_readings: Res<CurrentEvidenceReadings>,
    ghost_guess: Res<GhostGuess>,
    player_profile: Res<Persistent<PlayerProfileData>>,
    difficulty: Res<CurrentDifficulty>,
    mut walkie_play: ResMut<WalkiePlay>,
    time: Res<Time>,
) {
    const HIGH_CLARITY_THRESHOLD: f32 = 0.75;
    const PROMPT_DELAY_SECONDS: f32 = 20.0;

    // --- Part e: Handle Loss of Condition for Active Timer (run this first) ---
    if let Some((timed_evidence, _potential_ghost, _initial_ack_count, _detection_time)) =
        timer.data
    {
        let Some(ev_reading) = current_evidence_readings.get_reading(timed_evidence) else {
            return;
        };
        if ev_reading.clarity < HIGH_CLARITY_THRESHOLD {
            // info!("Clarity for timed evidence {:?} dropped. Clearing timer.", timed_evidence);
            timer.data = None;
        } else {
            // Re-simulate to check if it still leads to one ghost
            let mut simulated_found_evidences = ghost_guess.evidences_found.clone();
            simulated_found_evidences.insert(timed_evidence);
            let simulated_missing_evidences = ghost_guess.evidences_missing.clone();
            let mission_ghosts = difficulty.0.ghost_set.as_vec();
            let mut possible_ghosts_recheck = Vec::new();

            for ghost_candidate in mission_ghosts {
                let mut is_compatible = true;
                for found_ev in &simulated_found_evidences {
                    if !ghost_candidate.evidences().contains(found_ev) {
                        is_compatible = false;
                        break;
                    }
                }
                if !is_compatible {
                    continue;
                }
                for missing_ev in &simulated_missing_evidences {
                    if ghost_candidate.evidences().contains(missing_ev) {
                        is_compatible = false;
                        break;
                    }
                }
                if is_compatible {
                    possible_ghosts_recheck.push(ghost_candidate);
                }
            }

            if possible_ghosts_recheck.len() != 1 {
                // info!("Timed evidence {:?} no longer leads to a single ghost. Clearing timer.", timed_evidence);
                timer.data = None;
            }
        }
    }

    // --- Part b & c: Identify "Newly" High-Clarity Evidence and Simulate Ghost ID ---
    // Only try to set a new timer if one isn't already active
    if timer.data.is_none() {
        for current_ev_candidate in Evidence::all() {
            let Some(ev_reading) = current_evidence_readings.get_reading(current_ev_candidate)
            else {
                return;
            };

            if ev_reading.clarity >= HIGH_CLARITY_THRESHOLD {
                // Check if this evidence is already found in the journal
                if ghost_guess.evidences_found.contains(&current_ev_candidate) {
                    continue; // Already logged, not "new" for this purpose
                }

                let mut simulated_found_evidences = ghost_guess.evidences_found.clone();
                simulated_found_evidences.insert(current_ev_candidate);
                let simulated_missing_evidences = ghost_guess.evidences_missing.clone();
                let mission_ghosts = difficulty.0.ghost_set.as_vec();
                let mut possible_ghosts = Vec::new();

                for ghost_candidate in mission_ghosts {
                    let mut is_compatible = true;
                    for found_ev in &simulated_found_evidences {
                        if !ghost_candidate.evidences().contains(found_ev) {
                            is_compatible = false;
                            break;
                        }
                    }
                    if !is_compatible {
                        continue;
                    }
                    for missing_ev in &simulated_missing_evidences {
                        if ghost_candidate.evidences().contains(missing_ev) {
                            is_compatible = false;
                            break;
                        }
                    }
                    if is_compatible {
                        possible_ghosts.push(ghost_candidate);
                    }
                }

                if possible_ghosts.len() == 1 {
                    let identified_ghost = possible_ghosts[0];
                    let initial_ack_count = player_profile
                        .times_evidence_acknowledged_on_gear
                        .get(&current_ev_candidate)
                        .copied()
                        .unwrap_or(0);

                    // info!(
                    //     "Potential ID timer started for {:?} with ghost {:?}. Initial ack: {}",
                    //     current_ev_candidate,
                    //     identified_ghost.name(),
                    //     initial_ack_count
                    // );
                    timer.data = Some((
                        current_ev_candidate,
                        identified_ghost,
                        initial_ack_count,
                        time.elapsed_secs(),
                    ));
                    break; // Start timer for the first one found, then process it next frame
                }
            }
        }
    }

    // --- Part d: Check Active Timer and Trigger Walkie ---
    if let Some((timed_evidence, _potential_ghost, initial_ack_count, detection_time)) = timer.data
    {
        if time.elapsed_secs() - detection_time > PROMPT_DELAY_SECONDS {
            let current_ack_count = player_profile
                .times_evidence_acknowledged_on_gear
                .get(&timed_evidence)
                .copied()
                .unwrap_or(0);

            if current_ack_count == initial_ack_count {
                let event_triggered = walkie_play.set(
                    WalkieEvent::PotentialGhostIDWithNewEvidence,
                    time.elapsed_secs_f64(),
                );

                if event_triggered {
                    walkie_play.evidence_hinted_not_logged_via_walkie =
                        Some((timed_evidence, time.elapsed_secs_f64()));
                    // info!(
                    //     "Walkie prompt for potential ID triggered for {:?}. Ghost: {:?}",
                    //     timed_evidence,
                    //     potential_ghost.name()
                    // );
                    timer.data = None; // Clear timer after firing
                } else {
                    // info!(
                    //     "Walkie event for potential ID (timed_evidence: {:?}) not set (cooldown or other reason). Timer remains for now.",
                    //      timed_evidence
                    // );
                }
            } else {
                // info!(
                //     "Player acknowledged evidence {:?} on gear (current_ack_count: {}, initial_ack_count: {}). Clearing potential ID timer.",
                //     timed_evidence, current_ack_count, initial_ack_count
                // );
                timer.data = None; // Clear timer
            }
        } else {
            // Timer active, waiting for delay.
            // info!("Potential ID timer for {:?} active, waiting for delay. Time left: {:.1}s", timed_evidence, PROMPT_DELAY_SECONDS - (time.elapsed_secs() - detection_time));
        }
    }
}
