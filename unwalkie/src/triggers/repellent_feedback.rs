use bevy::prelude::*;
use bevy_platform::collections::{HashMap, HashSet};
use uncore::{
    components::{ghost_sprite::GhostSprite, repellent_particle::RepellentParticle},
    resources::ghost_guess::GhostGuess,
    states::AppState,
    types::{evidence::Evidence, ghost::types::GhostType},
};
use untruck::journal::ForceDiscardEvidenceEvent;
use unwalkiecore::{events::WalkieEvent, resources::WalkiePlay};

// Track which repellent types have already given hints this mission
#[derive(Resource, Default)]
struct RepellentHintsGiven {
    hints_given: HashSet<GhostType>,
}

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<RepellentHintsGiven>()
        .add_systems(Update, repellent_feedback_trigger_system);
}

fn repellent_feedback_trigger_system(
    time: Res<Time>,
    ghost_query: Query<&GhostSprite>,
    repellent_particle_query: Query<&RepellentParticle>,
    ghost_guess: Res<GhostGuess>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_force_discard: EventWriter<ForceDiscardEvidenceEvent>,
    mut hints_given: ResMut<RepellentHintsGiven>,
    app_state: Res<State<AppState>>,
) {
    // Reset hints when not in game
    if *app_state.get() != AppState::InGame {
        if !hints_given.hints_given.is_empty() {
            info!("RepellentFeedback: Resetting hints given due to leaving game");
            hints_given.hints_given.clear();
        }
        return;
    }

    // Count incorrect particles by repellent type
    let mut incorrect_particle_counts: HashMap<GhostType, usize> = HashMap::new();
    for particle in repellent_particle_query.iter() {
        if particle.hit_incorrect {
            *incorrect_particle_counts.entry(particle.class).or_insert(0) += 1;
        }
    }

    // Check if any repellent type has enough incorrect particles to trigger a hint
    for (repellent_type, count) in incorrect_particle_counts.iter() {
        if *count >= 50 && !hints_given.hints_given.contains(repellent_type) {
            // Find the ghost we're actually dealing with
            let Some(ghost_sprite) = ghost_query.iter().next() else {
                continue;
            };
            let real_ghost_type = ghost_sprite.class;

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
                    info!(
                        "RepellentFeedback: Sending hint for evidence {:?} and forcing discard (repellent {:?} vs ghost {:?}, {} particles)",
                        selected_evidence, repellent_type, real_ghost_type, count
                    );
                    ev_force_discard.write(ForceDiscardEvidenceEvent(selected_evidence));

                    // Mark this repellent type as having given a hint
                    hints_given.hints_given.insert(*repellent_type);
                    info!(
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
