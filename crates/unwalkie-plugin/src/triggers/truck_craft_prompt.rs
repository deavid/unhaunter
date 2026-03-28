use bevy::prelude::*;
use bevy_platform::collections::HashSet;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::types::gear::kind::GearKind;
use unghost_core::difficulty_ext::DifficultyGhostExt;
use uninvestigation_core::evidence::Evidence;
use uninvestigation_core::resources::current_evidence_readings::CurrentEvidenceReadings;
use uninvestigation_core::resources::ghost_guess::GhostGuess;
use unorchestrator_core::UIContextState;
use unplayer_core::components::MainPlayer;
use unwalkie_core::{events::walkie_types::WalkieEvent, resources::WalkiePlay};

fn trigger_almost_ready_to_craft_repellent_system(
    player_query: Query<&PlayerGear, With<MainPlayer>>,
    current_evidence_readings: Res<CurrentEvidenceReadings>,
    ghost_guess: Res<GhostGuess>,
    app_state: Res<State<UIContextState>>,
    difficulty: Res<CurrentDifficulty>,
    mut walkie_play: ResMut<WalkiePlay>,
    time: Res<Time>,
    mut clear_evidences: Local<HashSet<Evidence>>,
    mut repellent_crafted: Local<bool>,
    mut first_ready_time: Local<Option<f64>>,
    q_gear: Query<&GearKind>,
) {
    if *app_state != UIContextState::InGame {
        clear_evidences.clear();
        *repellent_crafted = false;
        *first_ready_time = None;
        return;
    }
    if *repellent_crafted {
        *first_ready_time = None;
        return;
    }
    // Check if player already has a repellent flask
    for player_gear in player_query.iter() {
        let check_gear = |entity: Entity| -> bool {
            if let Ok(kind) = q_gear.get(entity) {
                *kind == GearKind::RepellentFlask
            } else {
                false
            }
        };

        if player_gear.left_hand.map(check_gear).unwrap_or(false)
            || player_gear.right_hand.map(check_gear).unwrap_or(false)
            || player_gear.inventory.iter().any(|&e| check_gear(e))
        {
            // If player already has a repellent flask, no need to prompt to craft.
            *repellent_crafted = true;
            *first_ready_time = None;
            return;
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
    let mission_ghosts = difficulty.0.ghost_set().as_vec();
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
        *first_ready_time = None;
        return;
    }

    // All conditions met: trigger the prompt
    let current_time = time.elapsed_secs_f64();
    if first_ready_time.is_none() {
        *first_ready_time = Some(current_time);
    }

    if current_time - first_ready_time.unwrap() < 15.0 {
        return;
    }

    walkie_play.set(WalkieEvent::JournalPointsToOneGhostNoCraft, current_time);
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, trigger_almost_ready_to_craft_repellent_system);
}
