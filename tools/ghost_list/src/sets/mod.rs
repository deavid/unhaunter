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
    println!("\n✓ Basic parsing successful");
    eprintln!("Advanced set testing not yet implemented");
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

    // Show evidence distribution
    completion::analyze_evidence_distribution(&ghosts);

    eprintln!("\nAdvanced set analysis not yet implemented");
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
