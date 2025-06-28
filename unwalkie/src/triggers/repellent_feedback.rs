use bevy::prelude::*;
use bevy::time::Stopwatch;
use uncore::{
    components::{ghost_sprite::GhostSprite, repellent_particle::RepellentParticle},
    resources::ghost_guess::GhostGuess,
    types::evidence::Evidence,
};
use untruck::journal::ForceDiscardEvidenceEvent;
use unwalkiecore::{events::WalkieEvent, resources::WalkiePlay};

// Local timer to prevent spamming hints
#[derive(Resource, Default)]
struct RepellentFeedbackCooldownTimer(Timer);

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<RepellentFeedbackCooldownTimer>()
        .add_systems(Update, repellent_feedback_trigger_system);
}

fn repellent_feedback_trigger_system(
    time: Res<Time>,
    mut ghost_query: Query<&GhostSprite>,
    repellent_particle_query: Query<&RepellentParticle>,
    ghost_guess: Res<GhostGuess>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_force_discard: EventWriter<ForceDiscardEvidenceEvent>,
    mut repellent_miss_stopwatch: Local<Stopwatch>,
    mut hint_cooldown: ResMut<RepellentFeedbackCooldownTimer>,
) {
    if !hint_cooldown.0.tick(time.delta()).finished() {
        return; // Still in cooldown for giving any hint
    }

    for ghost_sprite in ghost_query.iter_mut() {
        if ghost_sprite.repellent_misses_delta > 0.0 {
            // Incorrect repellent was used this frame or is continuously being used
            repellent_miss_stopwatch.tick(time.delta());
        } else {
            // No incorrect repellent use this frame, reset the stopwatch if it was running
            if repellent_miss_stopwatch.elapsed_secs() > 0.0 {
                repellent_miss_stopwatch.reset();
            }
        }

        if repellent_miss_stopwatch.elapsed_secs() >= 10.0 {
            // Find the most recent repellent type that hit the ghost incorrectly
            let mut repellent_ghost_type = None;
            for particle in repellent_particle_query.iter() {
                if particle.hit_incorrect {
                    repellent_ghost_type = Some(particle.class);
                    break; // Use the first one found (most recent in query order)
                }
            }

            if let Some(repellent_type) = repellent_ghost_type {
                let real_ghost_type = ghost_sprite.class;
                let repellent_evidences = repellent_type.evidences();
                let real_evidences = real_ghost_type.evidences();

                // Find conflicting evidences (evidence the repellent suggests but the real ghost doesn't have)
                let mut conflicting_evidences: Vec<Evidence> = Vec::new();
                for evidence_candidate in Evidence::all() {
                    if repellent_evidences.contains(&evidence_candidate)
                        && !real_evidences.contains(&evidence_candidate)
                    {
                        conflicting_evidences.push(evidence_candidate);
                    }
                }

                if !conflicting_evidences.is_empty() {
                    // Strategy 1: Prefer evidence the player has marked as "found" but is actually wrong
                    let wrong_marked_evidences: Vec<Evidence> = conflicting_evidences
                        .iter()
                        .filter(|&evidence| ghost_guess.evidences_found.contains(evidence))
                        .copied()
                        .collect();

                    let selected_evidence = if !wrong_marked_evidences.is_empty() {
                        // Choose randomly from wrong marked evidences
                        let idx = (time.elapsed_secs_f64() * 1000.0) as usize
                            % wrong_marked_evidences.len();
                        wrong_marked_evidences[idx]
                    } else {
                        // Strategy 2: If none are marked as found, pick a random conflicting evidence
                        let idx = (time.elapsed_secs_f64() * 1000.0) as usize
                            % conflicting_evidences.len();
                        conflicting_evidences[idx]
                    };

                    // Construct the specific event with the evidence
                    let walkie_event = WalkieEvent::IncorrectRepellentHint(selected_evidence);

                    // Attempt to play the walkie event
                    if walkie_play.set(walkie_event.clone(), time.elapsed_secs_f64()) {
                        println!(
                            "RepellentFeedback: Sending hint for evidence {:?} and forcing discard",
                            selected_evidence
                        );
                        ev_force_discard.write(ForceDiscardEvidenceEvent(selected_evidence));

                        // Reset the stopwatch for this specific trigger
                        repellent_miss_stopwatch.reset();
                        repellent_miss_stopwatch.pause(); // Pause until next miss

                        // Reset and start the global hint cooldown for this system
                        hint_cooldown.0 = Timer::from_seconds(60.0, TimerMode::Once);

                        // Only one hint per trigger activation
                        return;
                    }
                } else {
                    // No conflicting evidence found - repellent and ghost have compatible evidence
                    // This shouldn't happen if repellent_misses_delta > 0, but reset anyway
                    repellent_miss_stopwatch.reset();
                    repellent_miss_stopwatch.pause();
                }
            } else {
                // No repellent particle with hit_incorrect found
                // Reset timer to prevent continuous trigger
                repellent_miss_stopwatch.reset();
                repellent_miss_stopwatch.pause();
            }
        }
    }
}
