use bevy::prelude::*;
use bevy_platform::collections::HashSet;
use uncore::difficulty::CurrentDifficulty;
use uncore::resources::current_evidence_readings::CurrentEvidenceReadings;
use uncore::resources::ghost_guess::GhostGuess;
use uncore::states::AppState;
use uncore::types::evidence::Evidence;
use uncore::types::gear_kind::GearKind;
use ungear::components::playergear::PlayerGear;
use unwalkiecore::{events::WalkieEvent, resources::WalkiePlay};

fn trigger_almost_ready_to_craft_repellent_system(
    player_query: Query<&PlayerGear>,
    current_evidence_readings: Res<CurrentEvidenceReadings>,
    ghost_guess: Res<GhostGuess>,
    app_state: Res<State<AppState>>,
    difficulty: Res<CurrentDifficulty>,
    mut walkie_play: ResMut<WalkiePlay>,
    time: Res<Time>,
    mut clear_evidences: Local<HashSet<Evidence>>,
    mut repellent_crafted: Local<bool>,
) {
    if *app_state != AppState::InGame {
        clear_evidences.clear();
        *repellent_crafted = false;
        return;
    }
    if *repellent_crafted {
        return;
    }
    // Check if player already has a repellent flask
    if let Ok(player_gear) = player_query.single() {
        for (gear, _epos) in player_gear.as_vec() {
            if gear.kind == GearKind::RepellentFlask {
                // If player already has a repellent flask, no need to prompt to craft.
                *repellent_crafted = true;
                return;
            }
        }
    }

    // Check if clear evidence uniquely identifies the correct ghost
    const HIGH_CLARITY_THRESHOLD: f32 = 0.25;

    // Collect all clear evidences
    for evidence in enum_iterator::all::<Evidence>() {
        if clear_evidences.contains(&evidence) {
            continue;
        }

        if current_evidence_readings.is_clearly_visible(evidence, HIGH_CLARITY_THRESHOLD) {
            clear_evidences.insert(evidence);
        }
    }

    // Collect all player-marked evidences
    for evidence in ghost_guess.evidences_found.iter() {
        clear_evidences.insert(*evidence);
    }

    // If no clear evidence, don't prompt
    if clear_evidences.is_empty() {
        return;
    }

    // Find which ghosts are compatible with the clear evidences
    let mission_ghosts = difficulty.0.ghost_set.as_vec();
    let mut compatible_ghosts = Vec::new();
    for ghost_type in mission_ghosts {
        let ghost_evidences = ghost_type.evidences();

        // Check if all clear evidences are compatible with this ghost
        let is_compatible = clear_evidences
            .iter()
            .all(|evidence| ghost_evidences.contains(evidence));

        if is_compatible {
            compatible_ghosts.push(ghost_type);
        }
    }

    if compatible_ghosts.len() != 1 {
        return;
    }

    // All conditions met: trigger the prompt
    walkie_play.set(
        WalkieEvent::JournalPointsToOneGhostNoCraft,
        time.elapsed_secs_f64(),
    );
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, trigger_almost_ready_to_craft_repellent_system);
}
