use bevy::prelude::*;
use bevy_platform::collections::{HashMap, HashSet};
use uncommon_states_core::UIContextState;
use unghost_core::components::presentation::repellent_particle::RepellentParticle;
use unghost_core::resources::signals::GhostHuntSignals;
use uninvestigation_core::evidence::Evidence;
use uninvestigation_core::ghost::GhostType;
use uninvestigation_core::resources::ghost_guess::GhostGuess;
use untruck_core::journal::ForceDiscardEvidenceEvent;
use unwalkie_core::{events::walkie_types::WalkieEvent, resources::WalkiePlay};

// Track which repellent types have already given hints this mission
#[derive(Resource, Default)]
struct RepellentHintsGiven {
    hints_given: HashSet<GhostType>,
    ready_to_play: HashSet<GhostType>, // Track which hints are ready to play when particle count drops
}

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<RepellentHintsGiven>()
        .add_systems(Update, repellent_feedback_trigger_system);
}

fn repellent_feedback_trigger_system(
    time: Res<Time>,
    hunt_signals: Res<GhostHuntSignals>,
    repellent_particle_query: Query<&RepellentParticle>,
    ghost_guess: Res<GhostGuess>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_force_discard: MessageWriter<ForceDiscardEvidenceEvent>,
    mut hints_given: ResMut<RepellentHintsGiven>,
    app_state: Res<State<UIContextState>>,
) {
    // Reset hints when not in game
    if *app_state.get() != UIContextState::InGame {
        if !hints_given.hints_given.is_empty() || !hints_given.ready_to_play.is_empty() {
            debug!("RepellentFeedback: Resetting hints given due to leaving game");
            hints_given.hints_given.clear();
            hints_given.ready_to_play.clear();
        }
        return;
    }

    // Count incorrect particles and total particles by repellent type
    let mut incorrect_particle_counts: HashMap<GhostType, usize> = HashMap::new();
    let mut total_particle_counts: HashMap<GhostType, usize> = HashMap::new();
    for particle in repellent_particle_query.iter() {
        *total_particle_counts.entry(particle.class).or_insert(0) += 1;
        if particle.hit_incorrect {
            *incorrect_particle_counts.entry(particle.class).or_insert(0) += 1;
        }
    }

    // First check: Mark repellent types as ready to play when they reach 50+ incorrect particles
    for (repellent_type, count) in incorrect_particle_counts.iter() {
        if *count >= 50
            && !hints_given.hints_given.contains(repellent_type)
            && !hints_given.ready_to_play.contains(repellent_type)
        {
            hints_given.ready_to_play.insert(*repellent_type);
            debug!(
                "RepellentFeedback: Marked {:?} as ready to play hint (50+ incorrect particles)",
                repellent_type
            );
        }
    }

    // Second check: Play hints for repellent types that are ready and now have < 5 total particles
    for repellent_type in hints_given.ready_to_play.clone() {
        let total_count = total_particle_counts.get(&repellent_type).unwrap_or(&0);

        if *total_count < 50 {
            // Find the ghost we're actually dealing with
            let Some(primary) = hunt_signals.primary.clone() else {
                continue;
            };
            let real_ghost_type = primary.class;

            let repellent_evidences = repellent_type.evidences();
            let real_evidences = real_ghost_type.evidences();

            // Find conflicting evidences (evidence the repellent suggests but the real ghost doesn't have)
            let conflicting_evidences: Vec<Evidence> = Evidence::all()
                .filter(|evidence| {
                    repellent_evidences.contains(evidence) && !real_evidences.contains(evidence)
                })
                .collect();

            if !conflicting_evidences.is_empty() {
                // Strategy 1: Prefer evidence the player has marked as "found" but is actually wrong
                let wrong_marked_evidences: Vec<Evidence> = conflicting_evidences
                    .iter()
                    .filter(|&evidence| ghost_guess.evidences_found.contains(evidence))
                    .copied()
                    .collect();

                let selected_evidence = if !wrong_marked_evidences.is_empty() {
                    // Choose randomly from wrong marked evidences
                    let idx =
                        (time.elapsed_secs_f64() * 1000.0) as usize % wrong_marked_evidences.len();
                    wrong_marked_evidences[idx]
                } else {
                    // Strategy 2: If none are marked as found, pick a random conflicting evidence
                    let idx =
                        (time.elapsed_secs_f64() * 1000.0) as usize % conflicting_evidences.len();
                    conflicting_evidences[idx]
                };

                // Construct the specific event with the evidence
                let walkie_event = WalkieEvent::IncorrectRepellentHint(selected_evidence);

                // Attempt to play the walkie event
                if walkie_play.set(walkie_event, time.elapsed_secs_f64()) {
                    debug!(
                        "RepellentFeedback: Sending hint for evidence {:?} and forcing discard (repellent {:?} vs ghost {:?}, {} total particles)",
                        selected_evidence, repellent_type, real_ghost_type, total_count
                    );
                    ev_force_discard.write(ForceDiscardEvidenceEvent(selected_evidence));

                    // Mark this repellent type as having given a hint and remove from ready_to_play
                    hints_given.hints_given.insert(repellent_type);
                    hints_given.ready_to_play.remove(&repellent_type);
                    debug!(
                        "RepellentFeedback: Marked hint as given for repellent {:?}",
                        repellent_type
                    );

                    // Only one hint per trigger activation
                    return;
                }
            }
        }
    }
}
