#![cfg(not(target_arch = "wasm32"))]

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn get_asset_types() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("fonts", vec!["ttf"]),
        ("img", vec!["png"]),
        ("maps", vec!["tmx", "tsx"]),
        ("music", vec!["ogg"]),
        ("sounds", vec!["ogg"]),
        ("walkie", vec!["ogg"]),
        ("manual", vec!["png"]),
        ("phrasebooks", vec!["ron"]),
        ("upscaled", vec!["png"]),
    ]
}

fn find_assets_directory() -> Result<PathBuf, String> {
    if let Ok(cwd) = env::current_dir() {
        let assets_path = cwd.join("assets");
        if assets_path.is_dir() {
            return Ok(assets_path);
        }
    }

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let assets_path = PathBuf::from(manifest_dir).join("assets");
        if assets_path.is_dir() {
            return Ok(assets_path);
        }
    }

    if let Ok(executable_path) = env::current_exe() {
        if let Some(executable_dir) = executable_path.parent() {
            let assets_path = executable_dir.join("assets");
            if assets_path.is_dir() {
                return Ok(assets_path);
            }
        }
    }

    Err("Could not find assets directory".into())
}

fn get_asset_list(assets_dir: &Path) -> Result<Vec<String>, String> {
    let mut list = vec![];

    for entry in WalkDir::new(assets_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let relative_path = path.strip_prefix(assets_dir).map_err(|e| e.to_string())?;
            list.push(relative_path.to_string_lossy().to_string());
        }
    }
    list.sort();
    Ok(list)
}

pub fn update_assetidx_files() -> Result<(), String> {
    let assets_dir = find_assets_directory()?;
    let asset_list = get_asset_list(&assets_dir)?;
    let asset_types = get_asset_types();

    for (folder_name, ext_list) in asset_types {
        for ext in &ext_list {
            let asset_list_path = assets_dir.join(format!("index/{folder_name}-{ext}.assetidx"));
            let mut expected_file_contents: String = asset_list
                .iter()
                .filter(|p| p.starts_with(folder_name) && p.ends_with(ext))
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .join("\n");
            expected_file_contents.push('\n');
            
            if let Ok(mut file) = File::open(&asset_list_path) {
                let mut buf = String::new();
                if file.read_to_string(&mut buf).is_ok() && buf == expected_file_contents {
                    continue;
                }
            }
            
            if let Some(parent) = asset_list_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create index directory: {}", e))?;
            }

            let mut asset_list_file =
                File::create(&asset_list_path).map_err(|e| format!("Failed to create assetidx: {}", e))?;

            asset_list_file
                .write_all(expected_file_contents.as_bytes())
                .map_err(|e| format!("Failed to write to assetidx: {}", e))?;
        }
    }
    Ok(())
}
