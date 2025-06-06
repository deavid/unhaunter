// filepath: /home/deavid/git/rust/unhaunter/tools/text_to_speech/walkie_voice_generator/src/codegen.rs
//! Module for Rust code generation logic
//!
//! Functions for generating Rust code from the manifest.

use crate::constants::GENERATED_ASSETS_DIR;
use crate::manifest_types::WalkieLineManifestEntry;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

// Helper function to read file content if it exists
fn read_file_if_exists(path: &Path) -> Result<Option<String>, anyhow::Error> {
    if path.exists() {
        fs::read_to_string(path)
            .map(Some)
            .map_err(|e| anyhow::anyhow!("Failed to read existing file {:?}: {}", path, e))
    } else {
        Ok(None)
    }
}

pub fn generate_rust_code(
    manifest: &HashMap<String, WalkieLineManifestEntry>,
    output_dir_str: &str,
    delete_unused: bool, // Added delete_unused flag
) -> Result<(), anyhow::Error> {
    let output_path = Path::new(output_dir_str);
    fs::create_dir_all(output_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create output directory for Rust code {}: {}",
            output_dir_str,
            e
        )
    })?;

    if delete_unused {
        println!(
            "Checking for unused Rust module files in {} to delete...",
            output_dir_str
        );
        let active_ron_sources: std::collections::HashSet<String> = manifest
            .values()
            .map(|entry| entry.ron_file_source.clone())
            .collect();

        let expected_module_filenames: std::collections::HashSet<String> = active_ron_sources
            .iter()
            .map(|ron_src| {
                format!(
                    "{}.rs",
                    ron_src.replace(".ron", "").to_lowercase().replace('-', "_")
                )
            })
            .collect();

        let mut deleted_rust_files_count = 0;
        for entry in fs::read_dir(output_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "rs" {
                        let file_name = path
                            .file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            .to_string();
                        if file_name != "mod.rs" && !expected_module_filenames.contains(&file_name)
                        {
                            println!("Deleting unused Rust module file: {:?}", path);
                            fs::remove_file(&path).map_err(|e| {
                                anyhow::anyhow!(
                                    "Failed to delete unused Rust file {:?}: {}",
                                    path,
                                    e
                                )
                            })?;
                            deleted_rust_files_count += 1;
                        }
                    }
                }
            }
        }
        if deleted_rust_files_count > 0 {
            println!(
                "Cleanup of {} unused Rust module files complete.",
                deleted_rust_files_count
            );
        } else {
            println!("No unused Rust module files found to delete.");
        }
    }

    let mut ron_to_lines: HashMap<String, Vec<WalkieLineManifestEntry>> = HashMap::new();
    for entry in manifest.values() {
        ron_to_lines
            .entry(entry.ron_file_source.clone())
            .or_default()
            .push(entry.clone());
    }

    // Sort ron_to_lines by RON filename for consistent mod.rs generation
    let mut sorted_ron_to_lines: Vec<_> = ron_to_lines.into_iter().collect();
    sorted_ron_to_lines.sort_by_key(|(ron_filename, _)| ron_filename.clone());

    let mut mod_rs_content = String::from(
        "// THIS FILE IS AUTOMATICALLY GENERATED BY THE WALKIE VOICE GENERATOR TOOL\n// DO NOT EDIT MANUALLY\n\n",
    );
    mod_rs_content.push_str("pub use unwalkie_types::WalkieTag;\n\n");

    // All concepts should implement the ConceptTrait, so collect them for mod.rs
    let mut concept_enums = Vec::new();

    for (ron_filename_str, entries_for_ron_file) in sorted_ron_to_lines {
        // Use sorted_ron_to_lines, removed mut for entries_for_ron_file
        let module_name_snake = ron_filename_str
            .replace(".ron", "")
            .to_lowercase()
            .replace('-', "_");

        mod_rs_content.push_str(&format!("pub mod {};\n", module_name_snake));

        let mut rs_file_content = String::from(
            "// THIS FILE IS AUTOMATICALLY GENERATED BY THE WALKIE VOICE GENERATOR TOOL\n// DO NOT EDIT MANUALLY\n\n",
        );
        rs_file_content.push_str("use unwalkie_types::{VoiceLineData, WalkieTag};\n\n");
        // Add import for ConceptTrait - this is essential for the trait implementation
        rs_file_content.push_str("use crate::ConceptTrait;\n\n");

        let mut concept_to_lines: HashMap<String, Vec<WalkieLineManifestEntry>> = HashMap::new();
        for entry in entries_for_ron_file {
            // entries_for_ron_file is Vec now
            concept_to_lines
                .entry(entry.conceptual_id.clone())
                .or_default()
                .push(entry);
        }

        // Sort concept_to_lines by concept_id for consistent enum variant and impl block ordering
        let mut sorted_concept_to_lines: Vec<_> = concept_to_lines.into_iter().collect();
        sorted_concept_to_lines.sort_by_key(|(concept_id, _)| concept_id.clone());

        let concept_enum_name = format!(
            "{}Concept",
            module_name_snake
                .split('_')
                .map(|s| {
                    let mut c = s.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<String>()
        );

        // Add the concept enum to the list for mod.rs
        concept_enums.push((module_name_snake.clone(), concept_enum_name.clone()));

        // Define the enum directly at the module level
        rs_file_content
            .push_str("/// Defines the different voice line concepts available in this module.\n");
        rs_file_content.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
        rs_file_content.push_str(&format!("pub enum {} {{\n", concept_enum_name));
        for (concept_id, _) in &sorted_concept_to_lines {
            // Iterate over sorted concepts for enum variants
            rs_file_content.push_str(&format!("    {},\n", concept_id));
        }
        rs_file_content.push_str("}\n\n");

        // Implement methods for the enum
        rs_file_content.push_str(&format!("impl {} {{\n", concept_enum_name));
        rs_file_content
            .push_str("    /// Retrieves a vector of `VoiceLineData` for this concept variant.\n");
        rs_file_content.push_str("    pub fn get_lines(&self) -> Vec<VoiceLineData> {\n");
        rs_file_content.push_str("        match self {\n");

        for (concept_id, mut lines_for_concept) in sorted_concept_to_lines {
            // Use sorted_concept_to_lines
            rs_file_content.push_str(&format!("            Self::{} => vec![\n", concept_id));
            lines_for_concept.sort_by_key(|l| l.line_index); // Sort lines by line_index

            for line_data in lines_for_concept {
                // Iterate over sorted lines
                rs_file_content.push_str("                VoiceLineData {\n");
                let asset_relative_ogg_path = Path::new(GENERATED_ASSETS_DIR)
                    .strip_prefix("assets/")
                    .unwrap_or_else(|_| Path::new(GENERATED_ASSETS_DIR))
                    .join(&line_data.ogg_path)
                    .to_str()
                    .unwrap()
                    .replace('\\', "/");

                rs_file_content.push_str(&format!(
                    "                    ogg_path: \"{}\".to_string(),\n",
                    asset_relative_ogg_path
                ));
                rs_file_content.push_str(&format!(
                    "                    subtitle_text: \"{}\".to_string(),\n",
                    line_data.subtitle_text.replace('"', "\\\"")
                ));

                rs_file_content.push_str("                    tags: vec![");
                // Ensure tags are sorted for consistent output
                let mut sorted_tags = line_data.tags.clone();
                sorted_tags.sort_by_key(|tag| format!("{:?}", tag)); // Sort tags alphabetically by their Debug representation
                for (tag_idx, tag) in sorted_tags.iter().enumerate() {
                    if tag_idx > 0 {
                        rs_file_content.push_str(", ");
                    }
                    rs_file_content.push_str(&format!("WalkieTag::{:?}", tag));
                }
                rs_file_content.push_str("],\n");

                rs_file_content.push_str(&format!(
                    "                    length_seconds: {},\n",
                    line_data.length_seconds
                ));
                rs_file_content.push_str("                },\n");
            }
            rs_file_content.push_str("            ],\n");
        }
        rs_file_content.push_str("        }\n"); // End of match
        rs_file_content.push_str("    }\n"); // End of get_lines method
        rs_file_content.push_str("}\n\n"); // End of impl block

        // Add implementation of ConceptTrait
        rs_file_content.push_str("// Auto-generated implementation of ConceptTrait\n");
        rs_file_content.push_str(&format!("impl ConceptTrait for {} {{\n", concept_enum_name));
        rs_file_content.push_str("    fn get_lines(&self) -> Vec<VoiceLineData> {\n");
        rs_file_content.push_str("        // Delegate to the generated get_lines method\n");
        rs_file_content.push_str("        self.get_lines()\n");
        rs_file_content.push_str("    }\n");
        rs_file_content.push_str("}\n");

        let rs_file_path = output_path.join(format!("{}.rs", module_name_snake));

        match read_file_if_exists(&rs_file_path)? {
            Some(existing_content) if existing_content == rs_file_content => {
                println!("Rust module {:?} is already up-to-date.", rs_file_path);
            }
            _ => {
                let mut rs_file = File::create(&rs_file_path).map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to create Rust module file {:?}: {}",
                        rs_file_path,
                        e
                    )
                })?;
                rs_file.write_all(rs_file_content.as_bytes()).map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to write to Rust module file {:?}: {}",
                        rs_file_path,
                        e
                    )
                })?;
                println!("Generated Rust module: {:?}", rs_file_path);
            }
        }
    }

    let mod_rs_path = output_path.join("mod.rs");
    match read_file_if_exists(&mod_rs_path)? {
        Some(existing_content) if existing_content == mod_rs_content => {
            println!("Rust mod.rs {:?} is already up-to-date.", mod_rs_path);
        }
        _ => {
            let mut mod_file = File::create(&mod_rs_path).map_err(|e| {
                anyhow::anyhow!("Failed to create mod.rs file {:?}: {}", mod_rs_path, e)
            })?;
            mod_file.write_all(mod_rs_content.as_bytes()).map_err(|e| {
                anyhow::anyhow!("Failed to write to mod.rs file {:?}: {}", mod_rs_path, e)
            })?;
            println!("Generated Rust mod.rs: {:?}", mod_rs_path);
        }
    }

    Ok(())
}
