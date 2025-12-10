use uncore::types::ghost::types::GhostType;

pub fn show_ghost_table(ghosts: &[GhostType]) {
    println!("| Ghost Type | Evidences |");
    println!("|-----------|-----------|");

    for ghost in ghosts {
        let evidence_names: Vec<_> = ghost.evidences().iter().map(|e| e.name()).collect();
        println!("| {} | {} |", ghost.name(), evidence_names.join(", "));
    }
}
