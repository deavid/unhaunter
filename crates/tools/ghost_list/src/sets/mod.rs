pub mod comparison;
pub mod completion;
pub mod optimization;
pub mod validation;

use crate::utils::parse_ghost_list;

pub fn test_set(ghost_names: &str) {
    let ghosts = parse_ghost_list(ghost_names);

    if ghosts.is_empty() {
        eprintln!("Error: No valid ghosts found in: {}", ghost_names);
        return;
    }

    println!(
        "Testing ghost set: {} ({} ghosts)",
        ghost_names,
        ghosts.len()
    );

    for ghost in &ghosts {
        let evidence_names: Vec<_> = ghost.evidences().iter().map(|e| e.name()).collect();
        println!("  {} - {}", ghost.name(), evidence_names.join(", "));
    }

    // TODO: Add actual set testing logic
    // println!("\n✓ Basic parsing successful");
    // eprintln!("Advanced set testing not yet implemented");

    println!("\n--- Set Uniqueness Preview (defaulting to min_evidence = 2) ---");
    validation::validate_uniqueness(&ghosts, 2); // Defaulting to 2 for a basic test, this also prints evidence summary.

    // For a more direct or differently formatted balance preview, we could call show_evidence_summary separately.
    // However, validate_uniqueness already includes this information.
    // To avoid redundancy for now, I'll rely on validate_uniqueness's output.
    // If a distinct "Balance Score" (e.g., a single metric) is needed later,
    // that would require a new function. The current "balance preview" is the evidence distribution table.
    println!("\n--- Evidence Balance (covered by Uniqueness Preview) ---");
    // validation::show_evidence_summary(&ghosts); // This could be called if we want it separate from validate_uniqueness

    // Conflict detection:
    // The `validate_uniqueness` function already identifies conflicts where
    // a given number of evidences (min_evidence) is not enough to distinguish
    // between ghosts in the set. This serves as a good proxy for conflict detection.
    // If we need more specific "pairwise conflict" (e.g. ghosts sharing X out of Y evidences),
    // that would be a separate function.

    println!("\n✓ Test set analysis complete.");
    println!(
        "  Note: 'Conflict detection' and 'Balance scoring' are based on uniqueness and evidence summary."
    );
}

pub fn analyze_set(ghost_names: &str) {
    let ghosts = parse_ghost_list(ghost_names);

    if ghosts.is_empty() {
        eprintln!("Error: No valid ghosts found in: {}", ghost_names);
        return;
    }

    println!(
        "Analyzing ghost set: {} ({} ghosts)",
        ghost_names,
        ghosts.len()
    );

    // Show evidence distribution (balance metrics)
    println!("\n--- Balance Metrics (Evidence Distribution) ---");
    completion::analyze_evidence_distribution(&ghosts);

    // Gap Analysis
    println!("\n--- Gap Analysis ---");
    let mut evidence_counts: std::collections::HashMap<uncore::types::evidence::Evidence, usize> =
        std::collections::HashMap::new();
    for ghost in &ghosts {
        for evidence in ghost.evidences() {
            *evidence_counts.entry(evidence).or_insert(0) += 1;
        }
    }

    let mut under_represented_evidence = Vec::new();
    for evidence_type in enum_iterator::all::<uncore::types::evidence::Evidence>() {
        let count = evidence_counts.get(&evidence_type).copied().unwrap_or(0);
        // Define "under-represented": present in 0 ghosts, or 1 ghost if set is larger than 2.
        if count == 0 || (count == 1 && ghosts.len() > 2) {
            under_represented_evidence.push(evidence_type);
        }
    }

    if under_represented_evidence.is_empty() {
        println!("No significant evidence gaps found based on current heuristics.");
    } else {
        println!("Found potentially under-represented evidences:");
        for evidence in &under_represented_evidence {
            println!(
                "  - {}: present in {} ghost(s) in the set.",
                evidence.name(),
                evidence_counts.get(evidence).copied().unwrap_or(0)
            );
        }

        // Recommendation Engine (Basic)
        println!("\n--- Recommendations to Fill Gaps ---");
        let mut recommendations_made = 0;
        for needed_evidence in &under_represented_evidence {
            if recommendations_made >= 3 {
                // Limit number of recommendations for brevity
                break;
            }
            // Find up to 1 candidate ghost for this specific missing/rare evidence
            let candidates = completion::find_completing_ghosts(
                &ghosts,
                Some(needed_evidence.name()), // Requires this specific evidence
                None,
                1, // Max 1 candidate per needed_evidence for this basic recommendation
            );

            if !candidates.is_empty() {
                for candidate in candidates {
                    if recommendations_made < 3 {
                        println!(
                            "  - Consider adding '{}' (has {} and other evidences: {}) to cover '{}'.",
                            candidate.name(),
                            needed_evidence.name(),
                            candidate
                                .evidences()
                                .iter()
                                .map(|e| e.name())
                                .filter(|&n| n != needed_evidence.name())
                                .collect::<Vec<_>>()
                                .join(", "),
                            needed_evidence.name()
                        );
                        recommendations_made += 1;
                    }
                }
            }
        }
        if recommendations_made == 0 {
            println!(
                "No simple ghost additions found to cover specific gaps with single candidates."
            );
        }
    }

    // eprintln!("\nAdvanced set analysis not yet implemented"); // Remove this line
    println!("\n✓ Set analysis complete.");
}

pub fn complete_set(
    ghost_names: &str,
    requires_evidence: Option<&str>,
    excludes_evidence: Option<&str>,
    max_candidates: usize,
) {
    let existing_ghosts = parse_ghost_list(ghost_names);

    if existing_ghosts.is_empty() {
        eprintln!("Error: No valid ghosts found in: {}", ghost_names);
        return;
    }

    println!(
        "Finding candidates to complete set: {} ({} ghosts)",
        ghost_names,
        existing_ghosts.len()
    );

    let candidates = completion::find_completing_ghosts(
        &existing_ghosts,
        requires_evidence,
        excludes_evidence,
        max_candidates,
    );

    completion::show_completion_results(&candidates, requires_evidence, excludes_evidence);
}

pub fn validate_set(ghost_names: &str, min_evidence: usize) {
    let ghosts = parse_ghost_list(ghost_names);

    if ghosts.is_empty() {
        eprintln!("Error: No valid ghosts found in: {}", ghost_names);
        return;
    }

    println!(
        "Validating ghost set: {} ({} ghosts)",
        ghost_names,
        ghosts.len()
    );
    println!(
        "Minimum evidence for unique identification: {}",
        min_evidence
    );

    validation::validate_uniqueness(&ghosts, min_evidence);
}
