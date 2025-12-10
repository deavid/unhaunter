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

    if size == 0 {
        eprintln!("Error: Set size must be greater than 0");
        return;
    }

    let balance_weight = balance_factor.unwrap_or(0.5); // Default 50% weight on balance
    let max_overlap_limit = max_overlap.unwrap_or(3); // Default max 3 shared evidences

    let all_ghosts: Vec<GhostType> = all::<GhostType>().collect();

    if size > all_ghosts.len() {
        eprintln!(
            "Error: Requested set size {} exceeds total number of ghosts {}",
            size,
            all_ghosts.len()
        );
        return;
    }

    println!(
        "Optimizing for evidence balance (weight: {:.1}) and overlap control (max: {} shared evidences)...",
        balance_weight, max_overlap_limit
    );

    let max_combinations_to_check = 50_000; // Reasonable limit for performance
    let mut scored_sets: Vec<(HashSet<GhostType>, f32)> = Vec::new();

    for (idx, ghost_combination) in all_ghosts.iter().combinations(size).enumerate() {
        if idx >= max_combinations_to_check {
            println!(
                "⚠️  Reached combination limit of {}. Results may not be exhaustive.",
                max_combinations_to_check
            );
            break;
        }

        let ghost_set: HashSet<GhostType> = ghost_combination.into_iter().cloned().collect();

        // Check overlap constraint first (hard constraint)
        if violates_overlap_constraint(&ghost_set, max_overlap_limit) {
            continue;
        }

        // Calculate optimization score
        let score = calculate_optimization_score(&ghost_set, balance_weight);

        if score > 0.0 {
            scored_sets.push((ghost_set, score));
        }
    }

    if scored_sets.is_empty() {
        println!("❌ No sets found meeting the optimization criteria.");
        return;
    }

    // Sort by score (higher is better)
    scored_sets.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Show top results
    let num_to_show = scored_sets.len().min(10);
    println!(
        "\n✅ Found {} optimized set(s). Showing top {}:",
        scored_sets.len(),
        num_to_show
    );

    println!("| Rank | Score | Ghosts | Balance Score | Overlap Score |");
    println!("|------|-------|--------|---------------|---------------|");

    for (rank, (ghost_set, total_score)) in scored_sets.iter().take(num_to_show).enumerate() {
        let ghost_names: Vec<&str> = ghost_set.iter().map(|g| g.name()).sorted().collect();
        let balance_score = calculate_balance_score(ghost_set);
        let overlap_score = calculate_overlap_score(ghost_set);

        println!(
            "| {} | {:.1} | {} | {:.1} | {:.1} |",
            rank + 1,
            total_score,
            ghost_names.join(", "),
            balance_score,
            overlap_score
        );
    }

    // Show detailed analysis for the best set
    if let Some((best_set, best_score)) = scored_sets.first() {
        println!("\n📋 Best Optimized Set Analysis:");
        println!("  Total Score: {:.1}", best_score);

        // Evidence analysis
        let all_evidences_in_set = get_all_evidences_in_set(best_set);
        println!(
            "  Evidence Types Used: {} of {}",
            all_evidences_in_set.len(),
            all::<Evidence>().count()
        );

        let evidence_names: Vec<&str> = all_evidences_in_set
            .iter()
            .map(|e| e.name())
            .sorted()
            .collect();
        println!("  Evidences: {}", evidence_names.join(", "));

        // Ghost details
        println!("\n  Ghost Details:");
        for ghost in best_set.iter().sorted_by_key(|g| g.name()) {
            let ghost_evidences: Vec<&str> = ghost.evidences().iter().map(|e| e.name()).collect();
            println!("    • {}: {}", ghost.name(), ghost_evidences.join(", "));
        }
    }
}

pub fn handle_diverse_set_command(size: usize, min_evidence_coverage: Option<usize>) {
    println!(
        "Generating diverse set of size {} (min_evidence_coverage: {:?})",
        size, min_evidence_coverage
    );

    let all_ghosts: Vec<GhostType> = all::<GhostType>().collect();
    let all_evidence_types: Vec<Evidence> = all::<Evidence>().collect();

    if size == 0 {
        eprintln!("Error: Set size must be greater than 0");
        return;
    }

    if size > all_ghosts.len() {
        eprintln!(
            "Error: Set size {} is larger than total available ghosts ({})",
            size,
            all_ghosts.len()
        );
        return;
    }

    let min_coverage = min_evidence_coverage.unwrap_or(all_evidence_types.len() / 2);
    println!(
        "Finding sets that cover at least {} unique evidence types...\n",
        min_coverage
    );

    let mut best_sets: Vec<(HashSet<GhostType>, usize)> = Vec::new();
    let mut max_diversity_found = 0;

    // We'll check a reasonable number of combinations to avoid performance issues
    let max_combinations_to_check = 10_000;
    let mut combinations_checked = 0;

    for combination in all_ghosts.iter().combinations(size) {
        if combinations_checked >= max_combinations_to_check {
            break;
        }
        combinations_checked += 1;

        let ghost_set: HashSet<GhostType> = combination.into_iter().cloned().collect();
        let diversity_score = calculate_evidence_diversity(&ghost_set);

        // Only consider sets that meet minimum coverage requirement
        if diversity_score >= min_coverage {
            if diversity_score > max_diversity_found {
                max_diversity_found = diversity_score;
                best_sets.clear();
                best_sets.push((ghost_set, diversity_score));
            } else if diversity_score == max_diversity_found {
                best_sets.push((ghost_set, diversity_score));
            }
        }
    }

    if best_sets.is_empty() {
        println!(
            "❌ No sets found that meet the minimum evidence coverage of {} types.",
            min_coverage
        );
        println!("💡 Try lowering --min-evidence-coverage or increasing --size");
        return;
    }

    // Sort by additional criteria (e.g., alphabetical for consistency)
    best_sets.sort_by(|(set_a, _), (set_b, _)| {
        let names_a: Vec<&str> = set_a.iter().map(|g| g.name()).sorted().collect();
        let names_b: Vec<&str> = set_b.iter().map(|g| g.name()).sorted().collect();
        names_a.cmp(&names_b)
    });

    // Show top results (limit to avoid spam)
    let max_results = 5.min(best_sets.len());

    println!(
        "✅ Found {} diverse set(s) with maximum coverage of {} evidence types:",
        best_sets.len(),
        max_diversity_found
    );
    println!("| Set | Ghosts | Evidence Types Covered |");
    println!("|-----|--------|-----------------------|");

    for (i, (ghost_set, diversity)) in best_sets.iter().take(max_results).enumerate() {
        let ghost_names: Vec<&str> = ghost_set.iter().map(|g| g.name()).sorted().collect();
        let covered_evidence = get_evidence_types_covered(ghost_set.clone());
        let evidence_names: Vec<&str> =
            covered_evidence.iter().map(|e| e.name()).sorted().collect();

        println!(
            "| {} | {} | {} ({}) |",
            i + 1,
            ghost_names.join(", "),
            evidence_names.join(", "),
            diversity
        );
    }

    if best_sets.len() > max_results {
        println!(
            "\n... and {} more sets with the same diversity score",
            best_sets.len() - max_results
        );
    }

    if combinations_checked >= max_combinations_to_check {
        println!(
            "\n⚠️  Note: Only checked {} combinations due to performance limits.",
            max_combinations_to_check
        );
        println!("Results may not be exhaustive for larger set sizes.");
    }
}

fn calculate_evidence_diversity(ghost_set: &HashSet<GhostType>) -> usize {
    let covered_evidence = get_evidence_types_covered(ghost_set.clone());
    covered_evidence.len()
}

fn get_evidence_types_covered(ghost_set: HashSet<GhostType>) -> HashSet<Evidence> {
    let mut covered_evidence = HashSet::new();
    for ghost in ghost_set {
        for evidence in ghost.evidences() {
            covered_evidence.insert(evidence);
        }
    }
    covered_evidence
}

pub fn handle_tutorial_set_command(size: usize, beginner_friendly: bool) {
    println!(
        "Generating tutorial set of size {} (beginner_friendly: {})",
        size, beginner_friendly
    );

    let all_ghosts: Vec<GhostType> = all::<GhostType>().collect();

    if size == 0 {
        eprintln!("Error: Set size must be greater than 0");
        return;
    }

    if size > all_ghosts.len() {
        eprintln!(
            "Error: Set size {} is larger than total available ghosts ({})",
            size,
            all_ghosts.len()
        );
        return;
    }

    // Define "beginner-friendly" criteria
    let beginner_evidences = if beginner_friendly {
        // These are relatively common and easy to understand evidence types
        vec![
            "EMF Level 5",
            "Freezing Temps",
            "Spirit Box",
            "UV Ectoplasm",
        ]
    } else {
        // Include all evidence types for advanced tutorial sets
        all::<Evidence>().map(|e| e.name()).collect()
    };

    println!(
        "Criteria: {} evidence types, uniqueness required",
        if beginner_friendly {
            "beginner-friendly"
        } else {
            "all"
        }
    );

    let mut suitable_sets: Vec<(HashSet<GhostType>, f32)> = Vec::new();
    let max_combinations_to_check = 5_000; // Reduced for tutorial sets
    let mut combinations_checked = 0;

    for combination in all_ghosts.iter().combinations(size) {
        if combinations_checked >= max_combinations_to_check {
            break;
        }
        combinations_checked += 1;

        let ghost_set: HashSet<GhostType> = combination.into_iter().cloned().collect();

        // Score the set for tutorial suitability
        let tutorial_score =
            calculate_tutorial_score(&ghost_set, &beginner_evidences, beginner_friendly);

        // Only consider sets with a reasonable score
        if tutorial_score > 0.0 {
            suitable_sets.push((ghost_set, tutorial_score));
        }
    }

    if suitable_sets.is_empty() {
        println!("❌ No suitable tutorial sets found with the given criteria.");
        println!("💡 Try adjusting --size or disable --beginner-friendly for more options");
        return;
    }

    // Sort by tutorial score (higher is better)
    suitable_sets.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let max_results = 5.min(suitable_sets.len());

    println!(
        "\n✅ Found {} suitable tutorial set(s):",
        suitable_sets.len()
    );
    println!("| Rank | Score | Ghosts | Key Evidence Types |");
    println!("|------|-------|--------|--------------------|");

    for (i, (ghost_set, score)) in suitable_sets.iter().take(max_results).enumerate() {
        let ghost_names: Vec<&str> = ghost_set.iter().map(|g| g.name()).sorted().collect();
        let covered_evidence = get_evidence_types_covered(ghost_set.clone());
        let beginner_evidence_count = covered_evidence
            .iter()
            .filter(|e| beginner_evidences.contains(&e.name()))
            .count();

        println!(
            "| {} | {:.1} | {} | {}/{} beginner-friendly |",
            i + 1,
            score,
            ghost_names.join(", "),
            beginner_evidence_count,
            beginner_evidences.len()
        );
    }

    if suitable_sets.len() > max_results {
        println!(
            "\n... and {} more suitable sets",
            suitable_sets.len() - max_results
        );
    }

    // Show details for the top set
    if let Some((best_set, _)) = suitable_sets.first() {
        println!("\n📋 Best Tutorial Set Details:");
        for ghost in best_set.iter().sorted_by_key(|g| g.name()) {
            let evidences: Vec<&str> = ghost.evidences().iter().map(|e| e.name()).collect();
            let beginner_count = evidences
                .iter()
                .filter(|e| beginner_evidences.contains(e))
                .count();
            println!(
                "  • {}: {} ({}/{} beginner-friendly)",
                ghost.name(),
                evidences.join(", "),
                beginner_count,
                evidences.len()
            );
        }
    }

    if combinations_checked >= max_combinations_to_check {
        println!(
            "\n⚠️  Note: Only checked {} combinations due to performance limits.",
            max_combinations_to_check
        );
    }
}

fn calculate_tutorial_score(
    ghost_set: &HashSet<GhostType>,
    beginner_evidences: &[&str],
    beginner_friendly: bool,
) -> f32 {
    let mut score = 0.0;

    // Check uniqueness first - essential for tutorials
    let all_evidence: HashSet<Evidence> = all::<Evidence>().collect();
    if !is_set_uniquely_identifiable(ghost_set, &all_evidence) {
        return 0.0; // Not suitable if not uniquely identifiable
    }

    score += 50.0; // Base score for uniqueness

    // Count beginner-friendly evidence coverage
    let covered_evidence = get_evidence_types_covered(ghost_set.clone());
    let beginner_evidence_count = covered_evidence
        .iter()
        .filter(|e| beginner_evidences.contains(&e.name()))
        .count();

    if beginner_friendly {
        // Higher score for more beginner-friendly evidence types
        score += (beginner_evidence_count as f32 / beginner_evidences.len() as f32) * 30.0;

        // Bonus if all beginner evidences are covered
        if beginner_evidence_count == beginner_evidences.len() {
            score += 20.0;
        }

        // Penalty for complex evidence types
        let complex_evidences = ["500+ cpm", "RL Presence", "Floating Orbs"];
        let complex_count = covered_evidence
            .iter()
            .filter(|e| complex_evidences.contains(&e.name()))
            .count();
        score -= complex_count as f32 * 5.0;
    } else {
        // For advanced tutorials, reward evidence diversity
        score += covered_evidence.len() as f32 * 3.0;
    }

    // Bonus for good evidence distribution across ghosts
    let evidence_per_ghost: Vec<usize> = ghost_set.iter().map(|g| g.evidences().len()).collect();
    let avg_evidence = evidence_per_ghost.iter().sum::<usize>() as f32 / ghost_set.len() as f32;

    // Prefer sets where ghosts have similar amounts of evidence (more balanced)
    let variance: f32 = evidence_per_ghost
        .iter()
        .map(|&count| (count as f32 - avg_evidence).powi(2))
        .sum::<f32>()
        / ghost_set.len() as f32;
    score -= variance; // Lower variance = higher score

    score.max(0.0)
}

// Helper functions for optimize-set command

fn violates_overlap_constraint(ghost_set: &HashSet<GhostType>, max_overlap: usize) -> bool {
    let ghosts: Vec<&GhostType> = ghost_set.iter().collect();

    // Check every pair of ghosts for evidence overlap
    for i in 0..ghosts.len() {
        for j in (i + 1)..ghosts.len() {
            let ghost1_evidence: HashSet<Evidence> = ghosts[i].evidences().into_iter().collect();
            let ghost2_evidence: HashSet<Evidence> = ghosts[j].evidences().into_iter().collect();

            let overlap_count = ghost1_evidence.intersection(&ghost2_evidence).count();
            if overlap_count > max_overlap {
                return true; // Constraint violated
            }
        }
    }
    false
}

fn calculate_optimization_score(ghost_set: &HashSet<GhostType>, balance_weight: f32) -> f32 {
    let balance_score = calculate_balance_score(ghost_set);
    let overlap_score = calculate_overlap_score(ghost_set);

    // Weighted combination of balance and overlap scores
    balance_weight * balance_score + (1.0 - balance_weight) * overlap_score
}

fn calculate_balance_score(ghost_set: &HashSet<GhostType>) -> f32 {
    // Calculate how evenly distributed the evidences are across the set
    let mut evidence_counts: HashMap<Evidence, usize> = HashMap::new();

    for ghost in ghost_set {
        for evidence in ghost.evidences() {
            *evidence_counts.entry(evidence).or_insert(0) += 1;
        }
    }

    if evidence_counts.is_empty() {
        return 0.0;
    }

    // Calculate the variance in evidence usage
    let total_evidence_instances: usize = evidence_counts.values().sum();
    let avg_usage = total_evidence_instances as f32 / evidence_counts.len() as f32;

    let variance: f32 = evidence_counts
        .values()
        .map(|&count| (count as f32 - avg_usage).powi(2))
        .sum::<f32>()
        / evidence_counts.len() as f32;

    // Convert variance to a score (lower variance = higher score)
    // Use exponential decay to heavily penalize high variance
    (-variance / 2.0).exp() * 100.0
}

fn calculate_overlap_score(ghost_set: &HashSet<GhostType>) -> f32 {
    let ghosts: Vec<&GhostType> = ghost_set.iter().collect();
    let mut total_overlap = 0;
    let mut total_pairs = 0;

    // Calculate average pairwise overlap
    for i in 0..ghosts.len() {
        for j in (i + 1)..ghosts.len() {
            let ghost1_evidence: HashSet<Evidence> = ghosts[i].evidences().into_iter().collect();
            let ghost2_evidence: HashSet<Evidence> = ghosts[j].evidences().into_iter().collect();

            let overlap_count = ghost1_evidence.intersection(&ghost2_evidence).count();
            total_overlap += overlap_count;
            total_pairs += 1;
        }
    }

    if total_pairs == 0 {
        return 100.0; // Single ghost has perfect overlap score
    }

    let avg_overlap = total_overlap as f32 / total_pairs as f32;

    // Score based on optimal overlap (around 2-3 shared evidences per pair)
    let optimal_overlap = 2.5;
    let deviation = (avg_overlap - optimal_overlap).abs();

    // Exponential decay for deviation from optimal
    (-deviation).exp() * 100.0
}

fn get_all_evidences_in_set(ghost_set: &HashSet<GhostType>) -> HashSet<Evidence> {
    let mut all_evidences = HashSet::new();

    for ghost in ghost_set {
        for evidence in ghost.evidences() {
            all_evidences.insert(evidence);
        }
    }

    all_evidences
}
