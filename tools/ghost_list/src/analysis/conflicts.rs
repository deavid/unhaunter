use crate::filtering::evidence_parser::parse_evidence_list;
use enum_iterator::all;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use uncore::types::{evidence::Evidence, ghost::types::GhostType};

// This command will find ghosts that become indistinguishable if only a certain subset of evidence is considered,
// or find evidence types that are "in conflict" (e.g., never appear together or always appear together).
// The DESIGN.md mentions:
// ghost_list conflicts --evidence "Freezing Temps,EMF Level 5"
// ghost_list conflicts --show-all

pub fn handle_conflicts_command(evidence_filter_str: Option<&str>, show_all: bool) {
    if let Some(filter_str) = evidence_filter_str {
        println!("Analyzing conflicts for evidence subset: {}", filter_str);
        analyze_subset_conflicts(filter_str);
    } else if show_all {
        println!("Analyzing all potential evidence conflicts in the ghost database...");
        analyze_all_conflicts();
    } else {
        println!("Please specify an evidence subset with --evidence or use --show-all.");
    }
}

fn analyze_subset_conflicts(evidence_filter_str: &str) {
    let filter_evidence = parse_evidence_list(evidence_filter_str);
    if filter_evidence.is_empty() {
        eprintln!(
            "Error: Could not parse any evidence from '{}'",
            evidence_filter_str
        );
        return;
    }

    let all_ghosts: Vec<GhostType> = all::<GhostType>().collect();

    // Find ghosts that match this evidence subset exactly
    let matching_ghosts: Vec<&GhostType> = all_ghosts
        .iter()
        .filter(|ghost| {
            let ghost_evidence: HashSet<Evidence> = ghost.evidences().into_iter().collect();
            let filter_set: HashSet<Evidence> = filter_evidence.iter().cloned().collect();
            filter_set.is_subset(&ghost_evidence)
        })
        .collect();

    if matching_ghosts.is_empty() {
        println!(
            "No ghosts found with evidence subset: {}",
            evidence_filter_str
        );
        return;
    }

    println!(
        "Found {} ghosts with evidence subset:",
        matching_ghosts.len()
    );

    // Group by identical evidence sets within this subset
    let mut evidence_groups: HashMap<Vec<Evidence>, Vec<&GhostType>> = HashMap::new();

    for ghost in &matching_ghosts {
        let ghost_evidence: HashSet<Evidence> = ghost.evidences().into_iter().collect();
        let filtered_evidence: Vec<Evidence> = filter_evidence
            .iter()
            .filter(|e| ghost_evidence.contains(e))
            .cloned()
            .collect();

        evidence_groups
            .entry(filtered_evidence)
            .or_default()
            .push(*ghost);
    }

    // Report conflicts (groups with multiple ghosts)
    let mut conflicts_found = false;
    for (evidence_set, ghosts) in evidence_groups {
        if ghosts.len() > 1 {
            conflicts_found = true;
            let evidence_names: Vec<&str> = evidence_set.iter().map(|e| e.name()).collect();
            let ghost_names: Vec<&str> = ghosts.iter().map(|g| g.name()).collect();
            println!(
                "⚠️  CONFLICT: {} ghosts share evidence [{}]: {}",
                ghosts.len(),
                evidence_names.join(", "),
                ghost_names.join(", ")
            );
        }
    }

    if !conflicts_found {
        println!("✅ No conflicts found within evidence subset");
    }
}

fn analyze_all_conflicts() {
    let all_ghosts: Vec<GhostType> = all::<GhostType>().collect();
    let mut issues_found = false;

    println!("\n## 1. Evidence Count Validation");
    println!("Checking that all ghosts have exactly 5 evidences...");

    for ghost in &all_ghosts {
        let evidence_count = ghost.evidences().len();
        if evidence_count != 5 {
            issues_found = true;
            println!(
                "⚠️  {} has {} evidences (expected 5): [{}]",
                ghost.name(),
                evidence_count,
                ghost.evidences().iter().map(|e| e.name()).join(", ")
            );
        }
    }

    println!("\n## 2. Duplicate Evidence Sets");
    println!("Checking for ghosts with identical evidence sets...");

    let mut evidence_groups: HashMap<Vec<Evidence>, Vec<&GhostType>> = HashMap::new();

    for ghost in &all_ghosts {
        let mut evidence_set: Vec<Evidence> = ghost.evidences().into_iter().collect();
        evidence_set.sort_by_key(|e| e.name()); // Sort for consistent comparison
        evidence_groups.entry(evidence_set).or_default().push(ghost);
    }

    for (evidence_set, ghosts) in evidence_groups {
        if ghosts.len() > 1 {
            issues_found = true;
            let evidence_names: Vec<&str> = evidence_set.iter().map(|e| e.name()).collect();
            let ghost_names: Vec<&str> = ghosts.iter().map(|g| g.name()).collect();
            println!(
                "⚠️  DUPLICATE: {} ghosts share identical evidence [{}]: {}",
                ghosts.len(),
                evidence_names.join(", "),
                ghost_names.join(", ")
            );
        }
    }

    println!("\n## 3. Evidence Distribution Analysis");
    println!("Checking for over/under-represented evidence types...");

    let mut evidence_counts: HashMap<Evidence, usize> = HashMap::new();
    for ghost in &all_ghosts {
        for evidence in ghost.evidences() {
            *evidence_counts.entry(evidence).or_insert(0) += 1;
        }
    }

    let total_ghosts = all_ghosts.len();
    let expected_per_evidence = (total_ghosts * 5) / 8; // Rough estimate: 5 evidences per ghost, 8 evidence types

    for evidence in all::<Evidence>() {
        let count = *evidence_counts.get(&evidence).unwrap_or(&0);
        let percentage = (count as f32 / total_ghosts as f32) * 100.0;

        if count == 0 {
            issues_found = true;
            println!("⚠️  UNUSED: {} appears in 0 ghosts", evidence.name());
        } else if count as f32 > expected_per_evidence as f32 * 1.5 {
            println!(
                "📊 OVERUSED: {} appears in {} ghosts ({:.1}%)",
                evidence.name(),
                count,
                percentage
            );
        } else if (count as f32) < (expected_per_evidence as f32 * 0.5) && count > 0 {
            println!(
                "📊 UNDERUSED: {} appears in {} ghosts ({:.1}%)",
                evidence.name(),
                count,
                percentage
            );
        }
    }

    if !issues_found {
        println!("✅ No critical conflicts found in the ghost database");
    } else {
        println!("\n💡 Consider reviewing the flagged issues above for game balance");
    }
}
