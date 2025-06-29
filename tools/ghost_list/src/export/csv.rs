use csv::Writer;
use serde::Serialize;
use uncore::types::{evidence::Evidence, ghost::types::GhostType};

#[derive(Serialize)]
struct GhostCsvRow<'a> {
    name: &'a str,
    evidence1: &'a str,
    evidence2: &'a str,
    evidence3: &'a str,
}

pub fn show_ghost_csv(ghosts: &[GhostType]) {
    let mut wtr = Writer::from_writer(std::io::stdout());

    // Write header manually if you want specific names
    // wtr.write_record(&["Ghost Name", "Evidence 1", "Evidence 2", "Evidence 3"]).unwrap();

    for ghost in ghosts {
        let evidences: Vec<&str> = ghost.evidences().iter().map(Evidence::name).collect();
        let row = GhostCsvRow {
            name: ghost.name(),
            evidence1: evidences.get(0).copied().unwrap_or(""),
            evidence2: evidences.get(1).copied().unwrap_or(""),
            evidence3: evidences.get(2).copied().unwrap_or(""),
        };
        if let Err(e) = wtr.serialize(row) {
            eprintln!("Error writing ghost {} to CSV: {}", ghost.name(), e);
        }
    }

    if let Err(e) = wtr.flush() {
        eprintln!("Error flushing CSV writer: {}", e);
    }
}
