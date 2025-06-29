use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use uncore::types::evidence::Evidence;
use uncore::types::ghost::types::GhostType;

pub fn validate_uniqueness(ghosts: &[GhostType], min_evidence: usize) {
    println!("\n## Uniqueness Validation");

    if ghosts.len() < 2 {
        println!("✅ Set with {} ghost(s) is trivially unique.", ghosts.len());
        return;
    }

    let mut conflicts = Vec::new();
    let mut total_combinations = 0;

    // Check all possible evidence combinations of min_evidence size
    let all_evidence: Vec<Evidence> = enum_iterator::all().collect();

    for evidence_combo in all_evidence.iter().combinations(min_evidence) {
        total_combinations += 1;
        let evidence_set: HashSet<Evidence> = evidence_combo.into_iter().cloned().collect();

        // Find ghosts that match this evidence combination
        let matching_ghosts: Vec<&GhostType> = ghosts
            .iter()
            .filter(|ghost| {
                let ghost_evidence = ghost.evidences();
                evidence_set
                    .iter()
                    .all(|evidence| ghost_evidence.contains(evidence))
            })
            .collect();

        if matching_ghosts.len() > 1 {
            let evidence_names: Vec<&str> = evidence_set.iter().map(|e| e.name()).collect();
            let ghost_names: Vec<&str> = matching_ghosts.iter().map(|g| g.name()).collect();

            conflicts.push((evidence_names.join(" + "), ghost_names));
        }
    }

    if conflicts.is_empty() {
        println!(
            "✅ Set is uniquely identifiable with {} evidence!",
            min_evidence
        );
        println!(
            "   Checked {} possible evidence combinations.",
            total_combinations
        );
    } else {
        println!(
            "❌ Found {} conflicts with {} evidence:",
            conflicts.len(),
            min_evidence
        );
        println!("| Evidence Combination | Conflicting Ghosts |");
        println!("|---------------------|-------------------|");

        for (evidence_combo, ghost_names) in &conflicts {
            println!("| {} | {} |", evidence_combo, ghost_names.join(", "));
        }

        println!("\n💡 Consider:");
        println!("   - Increasing minimum evidence to {}", min_evidence + 1);
        println!("   - Removing one ghost from each conflict");
        println!("   - Adding ghosts with different evidence patterns");
    }

    // Show evidence distribution
    show_evidence_summary(ghosts);
}

fn show_evidence_summary(ghosts: &[GhostType]) {
    let mut evidence_count: HashMap<Evidence, usize> = HashMap::new();

    for ghost in ghosts {
        for evidence in ghost.evidences() {
            *evidence_count.entry(evidence).or_insert(0) += 1;
        }
    }

    println!("\n## Evidence Summary");
    println!("| Evidence | Ghosts | Coverage |");
    println!("|----------|--------|----------|");

    for evidence in enum_iterator::all::<Evidence>() {
        let count = *evidence_count.get(&evidence).unwrap_or(&0);
        let coverage = if !ghosts.is_empty() {
            (count as f32 / ghosts.len() as f32) * 100.0
        } else {
            0.0
        };
        println!("| {} | {} | {:.1}% |", evidence.name(), count, coverage);
    }
}
