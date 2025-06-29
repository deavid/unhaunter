use crate::utils::ghost_parser::parse_ghost_list;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use uncore::types::{evidence::Evidence, ghost::types::GhostType};

// Helper to parse named ghost sets like "SetName:GhostA,GhostB,GhostC"
fn parse_named_ghost_set(set_str: &str) -> Result<(String, HashSet<GhostType>), String> {
    let parts: Vec<&str> = set_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid set format '{}'. Expected 'SetName:GhostA,GhostB'.",
            set_str
        ));
    }
    let name = parts[0].trim().to_string();
    if name.is_empty() {
        return Err("Set name cannot be empty.".to_string());
    }
    let ghosts = parse_ghost_list(parts[1]);
    if ghosts.is_empty() {
        return Err(format!("No valid ghosts found in set '{}'.", name));
    }
    Ok((name, ghosts.into_iter().collect()))
}

fn parse_multiple_named_sets(
    set_strs: &[String],
) -> Result<Vec<(String, HashSet<GhostType>)>, String> {
    let mut parsed_sets = Vec::new();
    for set_str in set_strs {
        match parse_named_ghost_set(set_str) {
            Ok(parsed_set) => parsed_sets.push(parsed_set),
            Err(e) => return Err(e),
        }
    }
    if parsed_sets.len() < 2 {
        return Err("At least two sets are required for comparison.".to_string());
    }
    Ok(parsed_sets)
}

pub fn handle_compare_sets_command(set_strs: Vec<String>) {
    println!("Comparing ghost sets: {:?}", set_strs);
    match parse_multiple_named_sets(&set_strs) {
        Ok(sets) => {
            // Display basic set information
            println!("\n## Set Overview");
            for (name, ghosts) in &sets {
                println!("Set '{}': {} ghosts", name, ghosts.len());
                for ghost in ghosts {
                    println!("  - {}", ghost.name());
                }
            }

            // Detailed comparison analysis
            perform_detailed_comparison(&sets);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub fn handle_overlap_analysis_command(set_strs: Vec<String>) {
    println!("Performing overlap analysis for sets: {:?}", set_strs);
    match parse_multiple_named_sets(&set_strs) {
        Ok(sets) => {
            // TODO: Implement overlap analysis.
            // Example: For two sets, show common ghosts, ghosts unique to set1, ghosts unique to set2.
            // For multiple sets, could show pairwise overlaps or overall common ghosts.
            if sets.len() == 2 {
                let (name1, set1_ghosts) = &sets[0];
                let (name2, set2_ghosts) = &sets[1];

                let intersection: HashSet<_> = set1_ghosts.intersection(set2_ghosts).collect();
                let unique_to_set1: HashSet<_> = set1_ghosts.difference(set2_ghosts).collect();
                let unique_to_set2: HashSet<_> = set2_ghosts.difference(set1_ghosts).collect();

                println!("\nOverlap between '{}' and '{}':", name1, name2);
                println!(
                    "  Common ghosts ({}): {}",
                    intersection.len(),
                    intersection
                        .iter()
                        .map(|g| g.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!(
                    "  Unique to '{}' ({}): {}",
                    name1,
                    unique_to_set1.len(),
                    unique_to_set1
                        .iter()
                        .map(|g| g.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!(
                    "  Unique to '{}' ({}): {}",
                    name2,
                    unique_to_set2.len(),
                    unique_to_set2
                        .iter()
                        .map(|g| g.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            } else {
                perform_multi_set_overlap_analysis(&sets);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub fn handle_merge_sets_command(set_strs: Vec<String>, optimize: bool) {
    println!(
        "Merging ghost sets: {:?} (optimize: {})",
        set_strs, optimize
    );
    match parse_multiple_named_sets(&set_strs) {
        Ok(sets) => {
            let mut merged_ghosts_set: HashSet<GhostType> = HashSet::new();
            for (_, ghosts) in sets {
                merged_ghosts_set.extend(ghosts);
            }
            println!(
                "\nMerged set contains {} unique ghosts:",
                merged_ghosts_set.len()
            );
            let mut sorted_merged_ghosts: Vec<_> = merged_ghosts_set.into_iter().collect();
            sorted_merged_ghosts.sort_by_key(|g| g.name());
            for ghost in sorted_merged_ghosts {
                println!("  - {}", ghost.name());
            }
            if optimize {
                eprintln!("Optimization of merged set not yet implemented.");
                // TODO: If optimize is true, could run some logic from `optimization.rs`
                // or a new balancing/uniqueness check on the merged_ghosts.
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub fn handle_diff_sets_command(old_set_str: String, new_set_str: String) {
    println!("Diffing sets: OLD='{}', NEW='{}'", old_set_str, new_set_str);
    match (
        parse_named_ghost_set(&old_set_str),
        parse_named_ghost_set(&new_set_str),
    ) {
        (Ok((old_name, old_ghosts)), Ok((new_name, new_ghosts))) => {
            let added_ghosts: HashSet<_> = new_ghosts.difference(&old_ghosts).collect();
            let removed_ghosts: HashSet<_> = old_ghosts.difference(&new_ghosts).collect();
            let common_ghosts: HashSet<_> = old_ghosts.intersection(&new_ghosts).collect();

            println!(
                "\nComparison between '{}' (old) and '{}' (new):",
                old_name, new_name
            );
            println!(
                "  Common ghosts ({}): {}",
                common_ghosts.len(),
                common_ghosts
                    .iter()
                    .map(|g| g.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!(
                "  Added to '{}' ({}): {}",
                new_name,
                added_ghosts.len(),
                added_ghosts
                    .iter()
                    .map(|g| g.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!(
                "  Removed from '{}' ({}): {}",
                old_name,
                removed_ghosts.len(),
                removed_ghosts
                    .iter()
                    .map(|g| g.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        (Err(e), _) => eprintln!("Error parsing old set: {}", e),
        (_, Err(e)) => eprintln!("Error parsing new set: {}", e),
    }
}

// New detailed comparison analysis function
fn perform_detailed_comparison(sets: &[(String, HashSet<GhostType>)]) {
    println!("\n## Detailed Comparison Analysis");

    // 1. Size comparison
    println!("\n### Set Sizes");
    let mut size_comparison: Vec<(&str, usize)> = sets
        .iter()
        .map(|(name, ghosts)| (name.as_str(), ghosts.len()))
        .collect();
    size_comparison.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by size descending

    for (name, size) in &size_comparison {
        println!("  {}: {} ghosts", name, size);
    }

    // 2. Evidence coverage comparison
    println!("\n### Evidence Coverage");
    for (name, ghosts) in sets {
        let all_evidences = get_all_evidences_in_set_from_hashset(ghosts);
        let evidence_names: Vec<&str> = all_evidences.iter().map(|e| e.name()).sorted().collect();
        println!(
            "  {}: {} evidence types - {}",
            name,
            all_evidences.len(),
            evidence_names.join(", ")
        );
    }

    // 3. Pairwise set overlaps
    if sets.len() >= 2 {
        println!("\n### Pairwise Overlaps");
        for i in 0..sets.len() {
            for j in (i + 1)..sets.len() {
                let (name1, set1) = &sets[i];
                let (name2, set2) = &sets[j];

                let intersection: HashSet<_> = set1.intersection(set2).collect();
                let union: HashSet<_> = set1.union(set2).collect();

                let jaccard_similarity = if union.is_empty() {
                    0.0
                } else {
                    intersection.len() as f32 / union.len() as f32
                };

                println!(
                    "  {} vs {}: {} common ghosts ({:.1}% Jaccard similarity)",
                    name1,
                    name2,
                    intersection.len(),
                    jaccard_similarity * 100.0
                );

                if !intersection.is_empty() {
                    let common_names: Vec<&str> =
                        intersection.iter().map(|g| g.name()).sorted().collect();
                    println!("    Common: {}", common_names.join(", "));
                }
            }
        }
    }

    // 4. Evidence balance comparison
    println!("\n### Evidence Balance Analysis");
    for (name, ghosts) in sets {
        let balance_score = calculate_balance_score_from_hashset(ghosts);
        println!(
            "  {}: Balance Score = {:.1}/100 (higher = more balanced)",
            name, balance_score
        );
    }

    // 5. Uniqueness analysis
    println!("\n### Uniqueness Analysis");
    for (name, ghosts) in sets {
        if ghosts.len() <= 1 {
            println!("  {}: Single ghost sets are always unique", name);
            continue;
        }

        let uniqueness_score = calculate_uniqueness_score(ghosts);
        println!(
            "  {}: Uniqueness Score = {:.1}/100 (higher = more distinguishable)",
            name, uniqueness_score
        );
    }

    // 6. Recommendations
    println!("\n### Recommendations");
    if sets.len() == 2 {
        let (name1, set1) = &sets[0];
        let (name2, set2) = &sets[1];

        if set1.len() != set2.len() {
            let (larger_name, smaller_name) = if set1.len() > set2.len() {
                (name1, name2)
            } else {
                (name2, name1)
            };
            println!(
                "  • Consider balancing set sizes: {} is larger than {}",
                larger_name, smaller_name
            );
        }

        let intersection_count = set1.intersection(set2).count();
        if intersection_count > set1.len() / 2 || intersection_count > set2.len() / 2 {
            println!("  • High overlap detected - consider diversifying ghost selection");
        }

        if intersection_count == 0 {
            println!("  • No overlap - sets are completely independent");
        }
    } else {
        println!(
            "  • For {} sets, consider analyzing pairwise similarities above",
            sets.len()
        );

        // Find most and least balanced sets
        let mut balance_scores: Vec<(String, f32)> = sets
            .iter()
            .map(|(name, ghosts)| (name.clone(), calculate_balance_score_from_hashset(ghosts)))
            .collect();
        balance_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        if let (Some(best), Some(worst)) = (balance_scores.first(), balance_scores.last()) {
            if balance_scores.len() > 1 && best.1 != worst.1 {
                println!(
                    "  • Most balanced: {} ({:.1}), Least balanced: {} ({:.1})",
                    best.0, best.1, worst.0, worst.1
                );
            }
        }
    }
}

// Helper functions for comparison analysis
fn get_all_evidences_in_set_from_hashset(ghost_set: &HashSet<GhostType>) -> HashSet<Evidence> {
    let mut all_evidences = HashSet::new();
    for ghost in ghost_set {
        for evidence in ghost.evidences() {
            all_evidences.insert(evidence);
        }
    }
    all_evidences
}

fn calculate_balance_score_from_hashset(ghost_set: &HashSet<GhostType>) -> f32 {
    let mut evidence_counts: HashMap<Evidence, usize> = HashMap::new();

    for ghost in ghost_set {
        for evidence in ghost.evidences() {
            *evidence_counts.entry(evidence).or_insert(0) += 1;
        }
    }

    if evidence_counts.is_empty() {
        return 0.0;
    }

    let total_evidence_instances: usize = evidence_counts.values().sum();
    let avg_usage = total_evidence_instances as f32 / evidence_counts.len() as f32;

    let variance: f32 = evidence_counts
        .values()
        .map(|&count| (count as f32 - avg_usage).powi(2))
        .sum::<f32>()
        / evidence_counts.len() as f32;

    (-variance / 2.0).exp() * 100.0
}

fn calculate_uniqueness_score(ghost_set: &HashSet<GhostType>) -> f32 {
    if ghost_set.len() <= 1 {
        return 100.0;
    }

    let ghosts: Vec<&GhostType> = ghost_set.iter().collect();
    let mut total_similarity = 0.0;
    let mut pair_count = 0;

    for i in 0..ghosts.len() {
        for j in (i + 1)..ghosts.len() {
            let evidence1: HashSet<Evidence> = ghosts[i].evidences().into_iter().collect();
            let evidence2: HashSet<Evidence> = ghosts[j].evidences().into_iter().collect();

            let intersection_size = evidence1.intersection(&evidence2).count();
            let union_size = evidence1.union(&evidence2).count();

            let similarity = if union_size > 0 {
                intersection_size as f32 / union_size as f32
            } else {
                0.0
            };

            total_similarity += similarity;
            pair_count += 1;
        }
    }

    if pair_count == 0 {
        return 100.0;
    }

    let avg_similarity = total_similarity / pair_count as f32;
    (1.0 - avg_similarity) * 100.0 // Lower similarity = higher uniqueness
}

fn perform_multi_set_overlap_analysis(sets: &[(String, HashSet<GhostType>)]) {
    println!("\n## Multi-Set Overlap Analysis");

    if sets.is_empty() {
        println!("No sets to analyze.");
        return;
    }

    // Show basic set info
    println!("\n### Set Overview");
    for (name, ghosts) in sets {
        println!("  '{}': {} ghosts", name, ghosts.len());
    }

    // Find ghosts that appear in all sets (universal intersection)
    println!("\n### Universal Intersection (ghosts in ALL sets)");
    let mut universal_intersection = sets[0].1.clone();
    for (_, ghost_set) in sets.iter().skip(1) {
        universal_intersection = universal_intersection
            .intersection(ghost_set)
            .cloned()
            .collect();
    }

    if universal_intersection.is_empty() {
        println!("  No ghosts appear in all {} sets", sets.len());
    } else {
        let names: Vec<&str> = universal_intersection
            .iter()
            .map(|g| g.name())
            .sorted()
            .collect();
        println!(
            "  {} ghost(s) appear in all sets: {}",
            universal_intersection.len(),
            names.join(", ")
        );
    }

    // Find ghosts that appear in any set (universal union)
    println!("\n### Universal Union (all unique ghosts across sets)");
    let mut universal_union = HashSet::new();
    for (_, ghost_set) in sets {
        universal_union = universal_union.union(ghost_set).cloned().collect();
    }
    println!(
        "  Total unique ghosts across all sets: {}",
        universal_union.len()
    );

    // Pairwise overlap matrix
    println!("\n### Pairwise Overlap Matrix");
    println!(
        "| Set A \\ Set B | {} |",
        sets.iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
    );
    println!("|{}|", vec!["---"; sets.len() + 1].join("|"));

    for (i, (name_i, set_i)) in sets.iter().enumerate() {
        let mut row = vec![name_i.clone()];

        for (j, (_, set_j)) in sets.iter().enumerate() {
            if i == j {
                row.push("—".to_string()); // Diagonal
            } else {
                let intersection_count = set_i.intersection(set_j).count();
                row.push(intersection_count.to_string());
            }
        }

        println!("| {} |", row.join(" | "));
    }

    // Ghost membership analysis
    println!("\n### Ghost Membership Analysis");
    let mut ghost_membership: HashMap<GhostType, Vec<String>> = HashMap::new();

    for (set_name, ghost_set) in sets {
        for ghost in ghost_set {
            ghost_membership
                .entry(*ghost)
                .or_default()
                .push(set_name.clone());
        }
    }

    // Group by membership count
    let mut by_membership_count: HashMap<usize, Vec<GhostType>> = HashMap::new();
    for (ghost, sets_containing) in &ghost_membership {
        by_membership_count
            .entry(sets_containing.len())
            .or_default()
            .push(*ghost);
    }

    for membership_count in (1..=sets.len()).rev() {
        if let Some(ghosts_in_n_sets) = by_membership_count.get(&membership_count) {
            if !ghosts_in_n_sets.is_empty() {
                let ghost_names: Vec<&str> =
                    ghosts_in_n_sets.iter().map(|g| g.name()).sorted().collect();

                if membership_count == sets.len() {
                    println!(
                        "  Ghosts in ALL {} sets ({}): {}",
                        membership_count,
                        ghosts_in_n_sets.len(),
                        ghost_names.join(", ")
                    );
                } else if membership_count == 1 {
                    println!(
                        "  Ghosts in only 1 set ({}): {}",
                        ghosts_in_n_sets.len(),
                        ghost_names.join(", ")
                    );
                } else {
                    println!(
                        "  Ghosts in exactly {} sets ({}): {}",
                        membership_count,
                        ghosts_in_n_sets.len(),
                        ghost_names.join(", ")
                    );
                }
            }
        }
    }

    // Jaccard similarity matrix
    println!("\n### Jaccard Similarity Matrix (0.0 = no overlap, 1.0 = identical)");
    println!(
        "| Set A \\ Set B | {} |",
        sets.iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
    );
    println!("|{}|", vec!["---"; sets.len() + 1].join("|"));

    for (i, (name_i, set_i)) in sets.iter().enumerate() {
        let mut row = vec![name_i.clone()];

        for (j, (_, set_j)) in sets.iter().enumerate() {
            if i == j {
                row.push("1.00".to_string()); // Diagonal - identical to self
            } else {
                let intersection_size = set_i.intersection(set_j).count();
                let union_size = set_i.union(set_j).count();
                let jaccard = if union_size > 0 {
                    intersection_size as f32 / union_size as f32
                } else {
                    0.0
                };
                row.push(format!("{:.2}", jaccard));
            }
        }

        println!("| {} |", row.join(" | "));
    }

    // Recommendations
    println!("\n### Recommendations");

    if universal_intersection.len() > sets.len() / 2 {
        println!("  • High universal overlap detected - consider diversifying ghost selection");
    }

    let total_unique = universal_union.len();
    let total_slots: usize = sets.iter().map(|(_, ghosts)| ghosts.len()).sum();
    let redundancy_ratio = total_slots as f32 / total_unique as f32;

    if redundancy_ratio > 2.0 {
        println!(
            "  • High redundancy ratio ({:.1}:1) - many ghosts appear in multiple sets",
            redundancy_ratio
        );
    } else if redundancy_ratio < 1.2 {
        println!(
            "  • Low redundancy ratio ({:.1}:1) - sets are highly independent",
            redundancy_ratio
        );
    }

    // Find most and least similar pairs
    let mut similarities = Vec::new();
    for i in 0..sets.len() {
        for j in (i + 1)..sets.len() {
            let (name_i, set_i) = &sets[i];
            let (name_j, set_j) = &sets[j];

            let intersection_size = set_i.intersection(set_j).count();
            let union_size = set_i.union(set_j).count();
            let jaccard = if union_size > 0 {
                intersection_size as f32 / union_size as f32
            } else {
                0.0
            };

            similarities.push(((name_i.clone(), name_j.clone()), jaccard));
        }
    }

    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    if let Some(((most_sim_a, most_sim_b), max_sim)) = similarities.first() {
        if *max_sim > 0.5 {
            println!(
                "  • Most similar pair: {} & {} (Jaccard: {:.2})",
                most_sim_a, most_sim_b, max_sim
            );
        }
    }

    if let Some(((least_sim_a, least_sim_b), min_sim)) = similarities.last() {
        if *min_sim < 0.1 {
            println!(
                "  • Most independent pair: {} & {} (Jaccard: {:.2})",
                least_sim_a, least_sim_b, min_sim
            );
        }
    }
}
