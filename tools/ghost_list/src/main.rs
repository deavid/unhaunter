use enum_iterator::all;
use std::collections::HashMap;
use uncore::types::evidence::Evidence;
use uncore::types::ghost::types::GhostType;

fn main() {
    // Print all ghosts with their evidences
    println!("| Ghost Type | Evidences |");
    println!("|-----------|-----------|");

    // Collect all ghosts into a vector
    let mut ghosts: Vec<GhostType> = all::<GhostType>().collect();
    // Sort the ghosts alphabetically by name
    ghosts.sort_by(|a, b| a.name().cmp(b.name()));

    for ghost in ghosts {
        let evidence_names: Vec<_> = ghost.evidences().iter().map(|e| e.name()).collect();

        println!("| {} | {} |", ghost.name(), evidence_names.join(", "));
    }

    // Calculate and print statistics
    let mut evidence_count: HashMap<Evidence, usize> = HashMap::new();
    let mut total_ghosts = 0;

    for ghost in all::<GhostType>() {
        total_ghosts += 1;
        for evidence in ghost.evidences() {
            *evidence_count.entry(evidence).or_insert(0) += 1;
        }
    }

    println!("\n## Evidence Statistics");
    println!("\nTotal ghosts: {}", total_ghosts);
    println!("\n| Evidence | Count | Percentage |");
    println!("|----------|-------|------------|");

    for evidence in all::<Evidence>() {
        let count = *evidence_count.get(&evidence).unwrap_or(&0);
        let percentage = (count as f32 / total_ghosts as f32) * 100.0;
        println!("| {} | {} | {:.1}% |", evidence.name(), count, percentage);
    }
}
