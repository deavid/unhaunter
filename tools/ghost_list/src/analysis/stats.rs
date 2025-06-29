use enum_iterator::all;
use std::collections::HashMap;
use uncore::types::evidence::Evidence;
use uncore::types::ghost::types::GhostType;
use crate::cli::OutputFormat;

pub fn show_stats(ghosts: &[GhostType], format: &OutputFormat) {
    match format {
        OutputFormat::Table => {
            // Calculate and print statistics
            let mut evidence_count: HashMap<Evidence, usize> = HashMap::new();
            let total_ghosts = ghosts.len();

            for ghost in ghosts {
                for evidence in ghost.evidences() {
                    *evidence_count.entry(evidence).or_insert(0) += 1;
                }
            }

            println!("## Evidence Statistics");
            if total_ghosts > 0 {
                println!("\nTotal ghosts: {}", total_ghosts);
                println!("\n| Evidence | Count | Percentage |");
                println!("|----------|-------|------------|");

                for evidence in all::<Evidence>() {
                    let count = *evidence_count.get(&evidence).unwrap_or(&0);
                    let percentage = if total_ghosts > 0 {
                        (count as f32 / total_ghosts as f32) * 100.0
                    } else {
                        0.0
                    };
                    println!("| {} | {} | {:.1}% |", evidence.name(), count, percentage);
                }
            } else {
                println!("\nNo ghosts match the current filters.");
            }
        }
        OutputFormat::Json => {
            eprintln!("JSON output not yet implemented");
        }
        OutputFormat::Csv => {
            eprintln!("CSV output not yet implemented");
        }
    }
}
