pub mod evidence_parser;

use uncore::types::ghost::types::GhostType;
use crate::cli::Cli;
use evidence_parser::parse_evidence_list;

pub fn apply_evidence_filters(ghosts: Vec<GhostType>, cli: &Cli) -> Vec<GhostType> {
    let mut filtered = ghosts;

    // Filter by has_evidence (alias for has_all)
    if let Some(evidence_str) = &cli.has_evidence.as_ref().or(cli.has_all.as_ref()) {
        let required_evidence = parse_evidence_list(evidence_str);
        filtered.retain(|ghost| {
            let ghost_evidence = ghost.evidences();
            required_evidence
                .iter()
                .all(|evidence| ghost_evidence.contains(evidence))
        });
    }

    // Filter by missing_evidence
    if let Some(evidence_str) = &cli.missing_evidence {
        let excluded_evidence = parse_evidence_list(evidence_str);
        filtered.retain(|ghost| {
            let ghost_evidence = ghost.evidences();
            excluded_evidence
                .iter()
                .all(|evidence| !ghost_evidence.contains(evidence))
        });
    }

    // Filter by has_any
    if let Some(evidence_str) = &cli.has_any {
        let any_evidence = parse_evidence_list(evidence_str);
        filtered.retain(|ghost| {
            let ghost_evidence = ghost.evidences();
            any_evidence
                .iter()
                .any(|evidence| ghost_evidence.contains(evidence))
        });
    }

    filtered
}
