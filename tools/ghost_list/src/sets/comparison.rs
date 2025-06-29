use crate::utils::ghost_parser::parse_ghost_list;
use uncore::types::ghost::types::GhostType;
use std::collections::HashSet;

// Helper to parse named ghost sets like "SetName:GhostA,GhostB,GhostC"
fn parse_named_ghost_set(set_str: &str) -> Result<(String, HashSet<GhostType>), String> {
    let parts: Vec<&str> = set_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid set format '{}'. Expected 'SetName:GhostA,GhostB'.",
            set_str
        ));
    }
    let name = parts[0].trim().to_string();
    if name.is_empty() {
        return Err("Set name cannot be empty.".to_string());
    }
    let ghosts = parse_ghost_list(parts[1]);
    if ghosts.is_empty() {
        return Err(format!("No valid ghosts found in set '{}'.", name));
    }
    Ok((name, ghosts.into_iter().collect()))
}

fn parse_multiple_named_sets(set_strs: &[String]) -> Result<Vec<(String, HashSet<GhostType>)>, String> {
    let mut parsed_sets = Vec::new();
    for set_str in set_strs {
        match parse_named_ghost_set(set_str) {
            Ok(parsed_set) => parsed_sets.push(parsed_set),
            Err(e) => return Err(e),
        }
    }
    if parsed_sets.len() < 2 {
        return Err("At least two sets are required for comparison.".to_string());
    }
    Ok(parsed_sets)
}


pub fn handle_compare_sets_command(set_strs: Vec<String>) {
    println!("Comparing ghost sets: {:?}", set_strs);
    match parse_multiple_named_sets(&set_strs) {
        Ok(sets) => {
            // TODO: Implement detailed comparison logic.
            // For now, just list them and their sizes.
            for (name, ghosts) in &sets {
                println!("Set '{}': {} ghosts", name, ghosts.len());
                for ghost in ghosts {
                    println!("  - {}", ghost.name());
                }
            }
            eprintln!("Detailed comparison logic not yet implemented.");
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub fn handle_overlap_analysis_command(set_strs: Vec<String>) {
    println!("Performing overlap analysis for sets: {:?}", set_strs);
     match parse_multiple_named_sets(&set_strs) {
        Ok(sets) => {
            // TODO: Implement overlap analysis.
            // Example: For two sets, show common ghosts, ghosts unique to set1, ghosts unique to set2.
            // For multiple sets, could show pairwise overlaps or overall common ghosts.
            if sets.len() == 2 {
                let (name1, set1_ghosts) = &sets[0];
                let (name2, set2_ghosts) = &sets[1];

                let intersection: HashSet<_> = set1_ghosts.intersection(set2_ghosts).collect();
                let unique_to_set1: HashSet<_> = set1_ghosts.difference(set2_ghosts).collect();
                let unique_to_set2: HashSet<_> = set2_ghosts.difference(set1_ghosts).collect();

                println!("\nOverlap between '{}' and '{}':", name1, name2);
                println!("  Common ghosts ({}): {}", intersection.len(), intersection.iter().map(|g| g.name()).collect::<Vec<_>>().join(", "));
                println!("  Unique to '{}' ({}): {}", name1, unique_to_set1.len(), unique_to_set1.iter().map(|g| g.name()).collect::<Vec<_>>().join(", "));
                println!("  Unique to '{}' ({}): {}", name2, unique_to_set2.len(), unique_to_set2.iter().map(|g| g.name()).collect::<Vec<_>>().join(", "));
            } else {
                eprintln!("Overlap analysis for more than 2 sets not yet implemented in detail. Basic parsing done.");
                 for (name, ghosts) in &sets {
                    println!("Set '{}': {} ghosts", name, ghosts.len());
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub fn handle_merge_sets_command(set_strs: Vec<String>, optimize: bool) {
    println!("Merging ghost sets: {:?} (optimize: {})", set_strs, optimize);
     match parse_multiple_named_sets(&set_strs) {
        Ok(sets) => {
            let mut merged_ghosts_set: HashSet<GhostType> = HashSet::new();
            for (_, ghosts) in sets {
                merged_ghosts_set.extend(ghosts);
            }
            println!("\nMerged set contains {} unique ghosts:", merged_ghosts_set.len());
            let mut sorted_merged_ghosts: Vec<_> = merged_ghosts_set.into_iter().collect();
            sorted_merged_ghosts.sort_by_key(|g| g.name());
            for ghost in sorted_merged_ghosts {
                println!("  - {}", ghost.name());
            }
            if optimize {
                eprintln!("Optimization of merged set not yet implemented.");
                // TODO: If optimize is true, could run some logic from `optimization.rs`
                // or a new balancing/uniqueness check on the merged_ghosts.
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub fn handle_diff_sets_command(old_set_str: String, new_set_str: String) {
    println!("Diffing sets: OLD='{}', NEW='{}'", old_set_str, new_set_str);
    match (parse_named_ghost_set(&old_set_str), parse_named_ghost_set(&new_set_str)) {
        (Ok((old_name, old_ghosts)), Ok((new_name, new_ghosts))) => {
            let added_ghosts: HashSet<_> = new_ghosts.difference(&old_ghosts).collect();
            let removed_ghosts: HashSet<_> = old_ghosts.difference(&new_ghosts).collect();
            let common_ghosts: HashSet<_> = old_ghosts.intersection(&new_ghosts).collect();

            println!("\nComparison between '{}' (old) and '{}' (new):", old_name, new_name);
            println!("  Common ghosts ({}): {}", common_ghosts.len(), common_ghosts.iter().map(|g| g.name()).collect::<Vec<_>>().join(", "));
            println!("  Added to '{}' ({}): {}", new_name, added_ghosts.len(), added_ghosts.iter().map(|g| g.name()).collect::<Vec<_>>().join(", "));
            println!("  Removed from '{}' ({}): {}", old_name, removed_ghosts.len(), removed_ghosts.iter().map(|g| g.name()).collect::<Vec<_>>().join(", "));

        }
        (Err(e), _) => eprintln!("Error parsing old set: {}", e),
        (_, Err(e)) => eprintln!("Error parsing new set: {}", e),
    }
}
