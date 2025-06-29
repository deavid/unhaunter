use serde::Serialize;
use uncore::types::{evidence::Evidence, ghost::types::GhostType};

#[derive(Serialize)]
struct GhostJson<'a> {
    name: &'a str,
    evidence: Vec<&'a str>,
}

pub fn show_ghost_json(ghosts: &[GhostType]) {
    let mut ghost_data: Vec<GhostJson> = Vec::new();
    for ghost in ghosts {
        ghost_data.push(GhostJson {
            name: ghost.name(),
            evidence: ghost.evidences().iter().map(Evidence::name).collect(),
        });
    }

    match serde_json::to_string_pretty(&ghost_data) {
        Ok(json_string) => println!("{}", json_string),
        Err(e) => eprintln!("Error serializing ghosts to JSON: {}", e),
    }
}
