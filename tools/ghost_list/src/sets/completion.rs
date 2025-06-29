use crate::filtering::evidence_parser::parse_evidence_list;
use enum_iterator::all;
use std::collections::HashMap;
use uncore::types::evidence::Evidence;
use uncore::types::ghost::types::GhostType;

pub fn analyze_evidence_distribution(ghosts: &[GhostType]) {
    let mut evidence_count: HashMap<Evidence, usize> = HashMap::new();

    for ghost in ghosts {
        for evidence in ghost.evidences() {
            *evidence_count.entry(evidence).or_insert(0) += 1;
        }
    }

    println!("\n## Evidence Distribution in Set");
    println!("| Evidence | Count | Percentage |");
    println!("|----------|-------|------------|");

    for evidence in all::<Evidence>() {
        let count = *evidence_count.get(&evidence).unwrap_or(&0);
        let percentage = if !ghosts.is_empty() {
            (count as f32 / ghosts.len() as f32) * 100.0
        } else {
            0.0
        };
        println!("| {} | {} | {:.1}% |", evidence.name(), count, percentage);
    }
}

pub fn find_completing_ghosts(
    existing_ghosts: &[GhostType],
    requires_evidence: Option<&str>,
    excludes_evidence: Option<&str>,
    max_candidates: usize,
) -> Vec<GhostType> {
    let existing_set: std::collections::HashSet<GhostType> =
        existing_ghosts.iter().cloned().collect();

    let required_evidence = requires_evidence
        .map(parse_evidence_list)
        .unwrap_or_default();

    let excluded_evidence = excludes_evidence
        .map(parse_evidence_list)
        .unwrap_or_default();

    all::<GhostType>()
        .filter(|ghost| !existing_set.contains(ghost)) // Not already in set
        .filter(|ghost| {
            // Must have all required evidence
            let ghost_evidence = ghost.evidences();
            required_evidence
                .iter()
                .all(|evidence| ghost_evidence.contains(evidence))
        })
        .filter(|ghost| {
            // Must not have any excluded evidence
            let ghost_evidence = ghost.evidences();
            excluded_evidence
                .iter()
                .all(|evidence| !ghost_evidence.contains(evidence))
        })
        .take(max_candidates)
        .collect()
}

pub fn show_completion_results(
    candidates: &[GhostType],
    requires_evidence: Option<&str>,
    excludes_evidence: Option<&str>,
) {
    if let Some(req) = requires_evidence {
        println!("Required evidence: {}", req);
    }
    if let Some(exc) = excludes_evidence {
        println!("Excluded evidence: {}", exc);
    }

    if candidates.is_empty() {
        println!("\n❌ No candidates found matching the criteria.");
        return;
    }

    println!("\n✅ Found {} candidate(s):", candidates.len());
    println!("| Ghost | Evidences |");
    println!("|-------|-----------|");

    for ghost in candidates {
        let evidence_names: Vec<_> = ghost.evidences().iter().map(|e| e.name()).collect();
        println!("| {} | {} |", ghost.name(), evidence_names.join(", "));
    }
}
