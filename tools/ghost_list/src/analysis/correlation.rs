use uncore::types::{evidence::Evidence, ghost::types::GhostType};
use enum_iterator::all;
use std::collections::HashMap;

// This command will analyze how often two evidences appear together.
// DESIGN.md mentions:
// ghost_list correlate --evidence "Freezing Temps" --with "EMF Level 5"

pub fn handle_correlation_command(evidence1_str: &str, evidence2_str: Option<&str>) {
    // Try to parse evidence1
    let evidence1 = match all::<Evidence>().find(|e| e.name().eq_ignore_ascii_case(evidence1_str)) {
        Some(e) => e,
        None => {
            eprintln!("Error: Could not parse primary evidence '{}'.", evidence1_str);
            return;
        }
    };

    let all_ghosts = all::<GhostType>().collect::<Vec<_>>();
    let total_ghosts = all_ghosts.len();

    if let Some(e2_str) = evidence2_str {
        // Correlate evidence1 with evidence2
        let evidence2 = match all::<Evidence>().find(|e| e.name().eq_ignore_ascii_case(e2_str)) {
            Some(e) => e,
            None => {
                eprintln!("Error: Could not parse secondary evidence '{}'.", e2_str);
                return;
            }
        };
        println!("Analyzing correlation between '{}' and '{}':", evidence1.name(), evidence2.name());

        let mut count_e1 = 0;
        let mut count_e2 = 0;
        let mut count_both = 0;
        let mut count_e1_not_e2 = 0;
        let mut count_e2_not_e1 = 0;
        let mut count_neither = 0;

        for ghost in &all_ghosts {
            let has_e1 = ghost.evidences().contains(&evidence1);
            let has_e2 = ghost.evidences().contains(&evidence2);

            if has_e1 { count_e1 += 1; }
            if has_e2 { count_e2 += 1; }

            if has_e1 && has_e2 {
                count_both += 1;
            } else if has_e1 && !has_e2 {
                count_e1_not_e2 += 1;
            } else if !has_e1 && has_e2 {
                count_e2_not_e1 += 1;
            } else {
                count_neither += 1;
            }
        }

        println!("\nTotal ghosts analyzed: {}", total_ghosts);
        println!("Ghosts with '{}': {} ({:.1}%)", evidence1.name(), count_e1, (count_e1 as f32 / total_ghosts as f32) * 100.0);
        println!("Ghosts with '{}': {} ({:.1}%)", evidence2.name(), count_e2, (count_e2 as f32 / total_ghosts as f32) * 100.0);
        println!("--------------------------------------------------");
        println!("Ghosts with BOTH '{}' and '{}': {} ({:.1}%)", evidence1.name(), evidence2.name(), count_both, (count_both as f32 / total_ghosts as f32) * 100.0);
        println!("Ghosts with '{}' but NOT '{}': {} ({:.1}%)", evidence1.name(), evidence2.name(), count_e1_not_e2, (count_e1_not_e2 as f32 / total_ghosts as f32) * 100.0);
        println!("Ghosts with '{}' but NOT '{}': {} ({:.1}%)", evidence2.name(), evidence1.name(), count_e2_not_e1, (count_e2_not_e1 as f32 / total_ghosts as f32) * 100.0);
        println!("Ghosts with NEITHER '{}' NOR '{}': {} ({:.1}%)", evidence1.name(), evidence2.name(), count_neither, (count_neither as f32 / total_ghosts as f32) * 100.0);

        // Simple correlation metric: P(A|B) = P(A and B) / P(B)
        // Conditional probability of having E1 given E2
        if count_e2 > 0 {
            println!("P({} | {}): {:.2}", evidence1.name(), evidence2.name(), count_both as f32 / count_e2 as f32);
        }
        // Conditional probability of having E2 given E1
        if count_e1 > 0 {
            println!("P({} | {}): {:.2}", evidence2.name(), evidence1.name(), count_both as f32 / count_e1 as f32);
        }


    } else {
        // Correlate evidence1 with all other evidences
        println!("Analyzing correlation of '{}' with all other evidences:", evidence1.name());

        let mut correlations: HashMap<Evidence, (usize, usize)> = HashMap::new(); // (count_both, count_other_evidence)
        let mut count_e1_total = 0;
        for ghost in &all_ghosts {
            if ghost.evidences().contains(&evidence1) {
                count_e1_total +=1;
                for other_evidence_type in all::<Evidence>() {
                    if other_evidence_type == evidence1 { continue; }
                    if ghost.evidences().contains(&other_evidence_type) {
                        let entry = correlations.entry(other_evidence_type).or_insert((0,0));
                        entry.0 += 1; // count_both with evidence1
                    }
                }
            }
            // Separately count total occurrences of other evidences
            for other_evidence_type in all::<Evidence>() {
                 if other_evidence_type == evidence1 { continue; }
                 if ghost.evidences().contains(&other_evidence_type) {
                    let entry = correlations.entry(other_evidence_type).or_insert((0,0));
                    entry.1 += 1; // count_other_evidence
                 }
            }
        }

        println!("\nTotal ghosts with '{}': {} out of {}", evidence1.name(), count_e1_total, total_ghosts);
        println!("\nCorrelation with other evidences (P(Other | {})):", evidence1.name());
        println!("| Other Evidence | P(Other|E1) | P(E1|Other) | Both | E1 Only | Other Only |");
        println!("|----------------|-------------|-------------|------|---------|------------|");

        let mut sorted_correlations: Vec<_> = correlations.into_iter().collect();
        sorted_correlations.sort_by_key(|(ev, _)| ev.name());

        for (other_evidence, (count_both_with_e1, count_other_total)) in sorted_correlations {
            let p_other_given_e1 = if count_e1_total > 0 { count_both_with_e1 as f32 / count_e1_total as f32 } else { 0.0 };
            let p_e1_given_other = if count_other_total > 0 { count_both_with_e1 as f32 / count_other_total as f32 } else { 0.0 };
            let e1_only = count_e1_total - count_both_with_e1; // E1 but not Other
            let other_only = count_other_total - count_both_with_e1; // Other but not E1

            println!(
                "| {:<14} | {:<11.2} | {:<11.2} | {:<4} | {:<7} | {:<10} |",
                other_evidence.name(),
                p_other_given_e1,
                p_e1_given_other,
                count_both_with_e1,
                e1_only,
                other_only
            );
        }
    }
}
