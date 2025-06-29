use uncore::types::{evidence::Evidence, ghost::types::GhostType};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use enum_iterator::all;

// This command will find unique evidence combinations that identify specific ghosts,
// or list all unique N-evidence combinations present in the game.
// DESIGN.md mentions:
// ghost_list unique-combinations
// ghost_list unique-combinations --min-evidence 2 --max-evidence 3

pub fn handle_unique_combinations_command(min_evidence: Option<usize>, max_evidence: Option<usize>) {
    let min_e = min_evidence.unwrap_or(1); // Default min 1 evidence
    let all_evidence_count = all::<Evidence>().count();
    let max_e = max_evidence.unwrap_or(all_evidence_count); // Default max all evidences a ghost can have

    if min_e == 0 || max_e == 0 {
        eprintln!("Error: min-evidence and max-evidence must be greater than 0.");
        return;
    }
    if min_e > max_e {
        eprintln!("Error: min-evidence cannot be greater than max-evidence.");
        return;
    }

    println!(
        "Finding unique evidence combinations ({} to {} evidences):",
        min_e, max_e
    );

    let all_ghosts = all::<GhostType>().collect::<Vec<_>>();
    let all_evidence_types = all::<Evidence>().collect::<Vec<_>>();

    let mut combination_to_ghosts_map: HashMap<Vec<Evidence>, Vec<GhostType>> = HashMap::new();

    for k in min_e..=max_e {
        if k > all_evidence_types.len() { continue; } // Cannot pick k items from less than k items

        for evidence_combo_indices in (0..all_evidence_types.len()).combinations(k) {
            let mut current_combo: Vec<Evidence> = evidence_combo_indices.iter().map(|&i| all_evidence_types[i]).collect();
            current_combo.sort_by_key(|e| e.name()); // Ensure consistent ordering for map key

            let mut matching_ghosts_for_this_combo = Vec::new();
            for ghost in &all_ghosts {
                let ghost_evidences: HashSet<Evidence> = ghost.evidences().into_iter().collect();
                // A ghost matches if it has *exactly* this combination of evidences, no more, no less,
                // OR if it has *at least* this combination (superset).
                // The typical interpretation for "unique combinations" is that this set of k evidences
                // points to one or more ghosts.
                // Let's assume it means: ghosts for whom this combination is their *complete* set of evidences, OR
                // ghosts that have *at least* these evidences.
                // The `validate_uniqueness` in `sets/validation.rs` implies the "at least" (superset) interpretation.
                // Let's stick to that: a ghost is matched if its evidences are a superset of current_combo.

                let current_combo_set: HashSet<Evidence> = current_combo.iter().cloned().collect();
                if current_combo_set.is_subset(&ghost_evidences) {
                    matching_ghosts_for_this_combo.push(ghost.clone());
                }
            }

            if !matching_ghosts_for_this_combo.is_empty() {
                // To only store combinations that uniquely identify one ghost:
                // if matching_ghosts_for_this_combo.len() == 1 {
                //    combination_to_ghosts_map.insert(current_combo, matching_ghosts_for_this_combo);
                // }
                // For now, let's store all combinations and the ghosts they map to.
                // The user can then see which ones are truly "unique" (map to 1 ghost).
                 combination_to_ghosts_map.insert(current_combo.clone(), matching_ghosts_for_this_combo);
            }
        }
    }

    if combination_to_ghosts_map.is_empty() {
        println!("No evidence combinations found matching the criteria within the ghost database.");
        return;
    }

    println!("\nFound {} combinations:", combination_to_ghosts_map.len());
    println!("| Evidence Combination | Ghost(s) Matched | Count | Notes |");
    println!("|----------------------|------------------|-------|-------|");

    // Sort the map for consistent output - by number of evidences, then by evidence names
    let mut sorted_combinations: Vec<_> = combination_to_ghosts_map.into_iter().collect();
    sorted_combinations.sort_by(|(combo_a, _), (combo_b, _)| {
        combo_a.len().cmp(&combo_b.len()).then_with(|| {
            let names_a = combo_a.iter().map(|e| e.name()).join(", ");
            let names_b = combo_b.iter().map(|e| e.name()).join(", ");
            names_a.cmp(&names_b)
        })
    });

    for (combo, ghosts) in sorted_combinations {
        let combo_str = combo.iter().map(|e| e.name()).join(" + ");
        let ghost_names = ghosts.iter().map(|g| g.name()).join(", ");
        let note = if ghosts.len() == 1 { "Unique" } else { "Shared" };
        println!("| {} | {} | {} | {} |", combo_str, ghost_names, ghosts.len(), note);
    }
}
