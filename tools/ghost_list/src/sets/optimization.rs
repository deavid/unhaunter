use crate::filtering::evidence_parser::parse_evidence_list;
// crate::utils::ghost_parser::parse_ghost_list; // Not used directly in this version
use enum_iterator::all;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use uncore::types::{evidence::Evidence, ghost::types::GhostType};

const MAX_COMBO_LIMIT_PER_PROFILE: usize = 10_000; // Max combinations to check per unwanted_evidence_profile
const MAX_UNWANTED_PROFILES_TO_EXPLORE: usize = 64; // Limit iterations on unwanted_evidence_bitset

// Adapted from uncore/src/utils/ghost_setfinder.rs
// Renamed to reflect its role in the CLI tool.
pub fn find_scored_ghost_sets(
    wanted_evidence_str: &str,
    num_ghosts_target: usize,
    max_results: usize,
) -> Vec<(HashSet<GhostType>, i32)> {
    let wanted_evidence_input = parse_evidence_list(wanted_evidence_str);
    if wanted_evidence_input.is_empty() {
        eprintln!(
            "Error: No valid wanted evidence provided or parsed from '{}'.",
            wanted_evidence_str
        );
        return Vec::new();
    }
    if num_ghosts_target == 0 {
        eprintln!("Error: Number of ghosts to find must be greater than 0.");
        return Vec::new();
    }
    let wanted_evidence_set: HashSet<Evidence> = wanted_evidence_input.into_iter().collect();

    let all_available_evidence: HashSet<Evidence> = all::<Evidence>().collect();
    let unwanted_evidence_pool: Vec<Evidence> = all_available_evidence
        .difference(&wanted_evidence_set)
        .cloned()
        .collect();

    let unwanted_evidence_count = unwanted_evidence_pool.len();
    let all_ghosts_vec = all::<GhostType>().collect::<Vec<GhostType>>();

    let min_presence = (num_ghosts_target / 2).max(1);
    let min_absence = (num_ghosts_target / 4).max(1);

    let mut results: Vec<(HashSet<GhostType>, i32)> = Vec::new();

    // Iterate through subsets of unwanted evidence to vary ghost selection profiles
    let num_unwanted_profiles_possible = 1 << unwanted_evidence_count; // 2^unwanted_evidence_count
    let unwanted_profiles_to_iterate =
        num_unwanted_profiles_possible.min(MAX_UNWANTED_PROFILES_TO_EXPLORE);

    // If num_unwanted_profiles_possible > MAX_UNWANTED_PROFILES_TO_EXPLORE, we might want to select profiles
    // strategically, e.g. those with fewer "required" unwanted evidences, or just iterate a fixed number.
    // For now, just take the first MAX_UNWANTED_PROFILES_TO_EXPLORE.

    for i in 0..unwanted_profiles_to_iterate {
        // This ensures we are not biased if we truncate the iterations.
        // A more sophisticated selection of bitsets could be used if needed.
        let unwanted_evidence_bitset =
            if num_unwanted_profiles_possible > MAX_UNWANTED_PROFILES_TO_EXPLORE {
                // Simple way to get a spread of bitsets if we're truncating.
                // This isn't perfect but tries to sample.
                // A truly random or representative sample would be more complex.
                (i * (num_unwanted_profiles_possible / MAX_UNWANTED_PROFILES_TO_EXPLORE.max(1)))
                    % num_unwanted_profiles_possible
            } else {
                i
            };

        let current_unwanted_profile_active: HashSet<Evidence> = unwanted_evidence_pool
            .iter()
            .enumerate()
            .filter_map(|(idx, evidence)| {
                if (unwanted_evidence_bitset >> idx) & 1 == 1 {
                    Some(*evidence)
                } else {
                    None
                }
            })
            .collect();

        let filtered_ghosts_for_profile: Vec<GhostType> = all_ghosts_vec
            .iter()
            .cloned()
            .filter(|ghost| {
                for evidence_in_pool in &unwanted_evidence_pool {
                    let ghost_has_it = ghost.evidences().contains(evidence_in_pool);
                    if current_unwanted_profile_active.contains(evidence_in_pool) {
                        // This evidence is "required active" among unselected
                        if !ghost_has_it {
                            return false;
                        }
                    } else {
                        // This evidence is "required inactive" among unselected
                        if ghost_has_it {
                            return false;
                        }
                    }
                }
                true
            })
            .collect();

        if filtered_ghosts_for_profile.len() < num_ghosts_target {
            continue;
        }

        let mut evidence_present_tally: HashMap<Evidence, usize> = HashMap::new();
        let mut evidence_absent_tally: HashMap<Evidence, usize> = HashMap::new();
        for ghost in &filtered_ghosts_for_profile {
            for evidence in &wanted_evidence_set {
                if ghost.evidences().contains(evidence) {
                    *evidence_present_tally.entry(*evidence).or_insert(0) += 1;
                } else {
                    *evidence_absent_tally.entry(*evidence).or_insert(0) += 1;
                }
            }
        }

        if wanted_evidence_set
            .iter()
            .any(|e| evidence_present_tally.get(e).unwrap_or(&0) < &min_presence)
        {
            continue;
        }
        if wanted_evidence_set
            .iter()
            .any(|e| evidence_absent_tally.get(e).unwrap_or(&0) < &min_absence)
        {
            continue;
        }

        let mut good_ghosts: HashSet<GhostType> = HashSet::new();
        let mut choice_ghosts: HashSet<GhostType> = HashSet::new();
        for &ghost in &filtered_ghosts_for_profile {
            let mut too_frequent = false;
            let mut too_infrequent = false;
            for &evidence in &wanted_evidence_set {
                if ghost.evidences().contains(&evidence)
                    && *evidence_present_tally.get(&evidence).unwrap_or(&0) > min_presence
                {
                    too_frequent = true;
                }
                if !ghost.evidences().contains(&evidence)
                    && *evidence_absent_tally.get(&evidence).unwrap_or(&0) > min_absence
                {
                    too_infrequent = true;
                }
            }
            if too_frequent || too_infrequent {
                choice_ghosts.insert(ghost);
            } else {
                good_ghosts.insert(ghost);
            }
        }

        let combinations_limit = MAX_COMBO_LIMIT_PER_PROFILE;

        if good_ghosts.len() >= num_ghosts_target {
            for combination in good_ghosts
                .iter()
                .cloned()
                .combinations(num_ghosts_target)
                .take(combinations_limit)
            {
                let ghost_set: HashSet<GhostType> = combination.into_iter().collect();
                if is_set_uniquely_identifiable(&ghost_set, &wanted_evidence_set) {
                    let score = score_ghost_set_performance(&ghost_set, &wanted_evidence_set);
                    results.push((ghost_set, score));
                }
            }
        } else if good_ghosts.len() < num_ghosts_target && !choice_ghosts.is_empty() {
            let remaining_needed = num_ghosts_target - good_ghosts.len();
            if choice_ghosts.len() >= remaining_needed {
                for choice_combination in choice_ghosts
                    .iter()
                    .cloned()
                    .combinations(remaining_needed)
                    .take(combinations_limit)
                {
                    let mut current_set = good_ghosts.clone();
                    current_set.extend(choice_combination);

                    // Re-check tallies for the combined set before scoring
                    let mut current_present_tally: HashMap<Evidence, usize> = HashMap::new();
                    let mut current_absent_tally: HashMap<Evidence, usize> = HashMap::new();
                    for ghost in &current_set {
                        for ev in &wanted_evidence_set {
                            if ghost.evidences().contains(ev) {
                                *current_present_tally.entry(*ev).or_insert(0) += 1;
                            } else {
                                *current_absent_tally.entry(*ev).or_insert(0) += 1;
                            }
                        }
                    }
                    if wanted_evidence_set
                        .iter()
                        .any(|e| current_present_tally.get(e).unwrap_or(&0) < &min_presence)
                    {
                        continue;
                    }
                    if wanted_evidence_set
                        .iter()
                        .any(|e| current_absent_tally.get(e).unwrap_or(&0) < &min_absence)
                    {
                        continue;
                    }

                    if is_set_uniquely_identifiable(&current_set, &wanted_evidence_set) {
                        let score = score_ghost_set_performance(&current_set, &wanted_evidence_set);
                        results.push((current_set, score));
                    }
                }
            }
        }
    }

    results.sort_by_key(|&(_, score)| std::cmp::Reverse(score)); // Sort descending by score
    results.dedup_by_key(|(set, _)| {
        let mut sorted_set_vec: Vec<_> = set.iter().map(|g| g.name()).collect();
        sorted_set_vec.sort();
        sorted_set_vec
    });
    results.truncate(max_results);
    results
}

fn is_set_uniquely_identifiable(
    ghost_set: &HashSet<GhostType>,
    fingerprint_evidences: &HashSet<Evidence>, // Evidences to use for creating the fingerprint
) -> bool {
    if ghost_set.is_empty() {
        return true;
    }
    if fingerprint_evidences.is_empty() {
        return ghost_set.len() <= 1;
    } // if no evidence, only 1 ghost can be "unique"

    // Use a sorted list of the fingerprint_evidences to ensure consistent bitmask construction
    let sorted_fingerprint_evidences: Vec<Evidence> = fingerprint_evidences
        .iter()
        .cloned()
        .sorted_by_key(|e| e.name())
        .collect();

    let unique_fingerprints: HashSet<u64> = ghost_set
        .iter()
        .map(|ghost| {
            let mut bitset: u64 = 0;
            for (idx, evidence_type) in sorted_fingerprint_evidences.iter().enumerate() {
                if ghost.evidences().contains(evidence_type) {
                    bitset |= 1 << idx;
                }
            }
            bitset
        })
        .collect();
    unique_fingerprints.len() == ghost_set.len()
}

fn score_ghost_set_performance(
    ghost_set: &HashSet<GhostType>,
    scoring_evidences: &HashSet<Evidence>, // Evidences to consider for scoring
) -> i32 {
    if scoring_evidences.is_empty() || ghost_set.is_empty() {
        return 0;
    }

    let mut score = 200; // Base score, higher to allow for more nuanced subtractions

    let mut current_evidence_counts: HashMap<Evidence, usize> = HashMap::new();
    for &evidence in scoring_evidences {
        current_evidence_counts.insert(evidence, 0);
    }
    for ghost in ghost_set {
        for &evidence_type in scoring_evidences {
            if ghost.evidences().contains(&evidence_type) {
                *current_evidence_counts.get_mut(&evidence_type).unwrap() += 1;
            }
        }
    }

    // 1. Penalize for uneven distribution of scoring evidences
    let num_scoring_evidences = scoring_evidences.len();
    let total_instances_of_scoring_evidences = current_evidence_counts.values().sum::<usize>();
    let mean_count_per_evidence =
        total_instances_of_scoring_evidences as f64 / num_scoring_evidences as f64;

    let sum_sq_deviations = current_evidence_counts
        .values()
        .map(|&count| (count as f64 - mean_count_per_evidence).powi(2))
        .sum::<f64>();
    let std_dev = (sum_sq_deviations / num_scoring_evidences as f64).sqrt();
    score -= (std_dev * 10.0).round() as i32; // Penalty proportional to std deviation

    // 2. Penalize deviation from an "ideal" total count of evidence instances
    // Ideal: each ghost has about 2/3 of the scoring_evidences.
    let ideal_total_evidence_instances =
        ghost_set.len() as f64 * num_scoring_evidences as f64 * (2.0 / 3.0);
    score -= ((total_instances_of_scoring_evidences as f64 - ideal_total_evidence_instances).abs()
        * 2.0)
        .round() as i32;

    // 3. Bonus for each evidence type being present at least once (coverage)
    let covered_evidence_types = current_evidence_counts
        .values()
        .filter(|&&count| count > 0)
        .count();
    score += (covered_evidence_types * 5) as i32;

    // 4. Small penalty if any scoring evidence is completely missing
    if covered_evidence_types < num_scoring_evidences {
        score -= ((num_scoring_evidences - covered_evidence_types) * 10) as i32;
    }

    score.max(0) // Ensure score is not negative
}

pub fn handle_find_sets_command(target_evidence_str: &str, size: usize, max_results: usize) {
    println!(
        "Finding optimal sets of size {} for target evidence: '{}' (showing top {})",
        size, target_evidence_str, max_results
    );

    let scored_sets = find_scored_ghost_sets(target_evidence_str, size, max_results);

    if scored_sets.is_empty() {
        println!(
            "\nNo suitable ghost sets found matching the criteria after exploring possibilities."
        );
        println!(
            "Consider trying with different target evidence, set size, or if the game's ghost data allows for such a set."
        );
        return;
    }

    println!(
        "\nTop {} Optimal Ghost Set(s) Found (or fewer if not enough unique sets):",
        scored_sets.len()
    );
    println!("| Rank | Score | Ghosts in Set (Evidences) |");
    println!("|------|-------|---------------------------|");

    for (i, (ghost_set, score)) in scored_sets.iter().enumerate() {
        let mut ghost_details_parts = Vec::new();
        let mut sorted_ghost_set: Vec<_> = ghost_set.iter().collect();
        sorted_ghost_set.sort_by_key(|g| g.name());

        for ghost_in_set in sorted_ghost_set {
            let evidences_str = ghost_in_set.evidences().iter().map(|e| e.name()).join("/");
            ghost_details_parts.push(format!("{} ({})", ghost_in_set.name(), evidences_str));
        }

        println!(
            "| {:<4} | {:<5} | {} |",
            i + 1,
            score,
            ghost_details_parts.join(", ")
        );
    }
    println!(
        "\nNote: Higher scores are generally better. Scoring considers evidence balance and coverage within the set based on target evidences."
    );
}

// TODO: Implement logic for other optimization commands:
// `optimize-set --size X --balance-factor Y --max-overlap Z`
// `diverse-set --size X --min-evidence-coverage Y`
// `tutorial-set --beginner-friendly --size X`
// These will require new functions or significant modifications to `find_scored_ghost_sets`
// and potentially new scoring functions.
// - `optimize-set` might adjust scoring weights or add new scoring components.
// - `diverse-set` would primarily score based on the count of unique evidences present in the set.
// - `tutorial-set` might filter for ghosts with "easy" or "common" evidences and prioritize simpler sets.Tool output for `overwrite_file_with_block`:

pub fn handle_optimize_set_command(
    size: usize,
    balance_factor: Option<f32>,
    max_overlap: Option<usize>,
) {
    println!(
        "Generating optimized set of size {} (balance_factor: {:?}, max_overlap: {:?})",
        size, balance_factor, max_overlap
    );
    eprintln!("Optimize-set command logic not yet implemented.");
    // TODO: Implement logic for optimize-set.
    // This would likely iterate through combinations of ghosts of `size`.
    // A specialized scoring function would be needed that incorporates `balance_factor`
    // (e.g., how much to weigh even evidence distribution) and `max_overlap`
    // (e.g., penalizing sets where ghosts share too many evidences, or too many ghosts share the same evidence).
}

pub fn handle_diverse_set_command(size: usize, min_evidence_coverage: Option<usize>) {
    println!(
        "Generating diverse set of size {} (min_evidence_coverage: {:?})",
        size, min_evidence_coverage
    );
    eprintln!("Diverse-set command logic not yet implemented.");
    // TODO: Implement logic for diverse-set.
    // This would iterate through combinations of ghosts of `size`.
    // Scoring would prioritize maximizing the number of unique evidence types present across all ghosts in the set.
    // `min_evidence_coverage` would be a threshold for this score or a filter.
}

pub fn handle_tutorial_set_command(size: usize, beginner_friendly: bool) {
    println!(
        "Generating tutorial set of size {} (beginner_friendly: {})",
        size, beginner_friendly
    );
    eprintln!("Tutorial-set command logic not yet implemented.");
    // TODO: Implement logic for tutorial-set.
    // "beginner_friendly" is subjective. Could mean:
    // - Prioritizing ghosts with very common / distinct evidences.
    // - Ensuring high identifiability (e.g., using `is_set_uniquely_identifiable` with a low evidence count).
    // - Could use a predefined list of "easy" evidences or ghost characteristics.
}
