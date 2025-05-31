#![cfg(not(target_arch = "wasm32"))]

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{Read, Write};
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
        ("phrasebooks", vec!["yaml"]),
    ]
}

fn get_asset_list() -> Result<Vec<String>> {
    let mut list = vec![];
    let assets_dir = crate::utils::find_assets_directory().context("Assets directory not found")?;

    for entry in WalkDir::new(&assets_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let relative_path = path.strip_prefix(&assets_dir)?;
            list.push(relative_path.to_string_lossy().to_string());
        }
    }
    list.sort();
    Ok(list)
}

pub fn update_assetidx_files() -> Result<()> {
    let asset_list = get_asset_list()?;
    let asset_types = get_asset_types();
    let assets_dir = crate::utils::find_assets_directory().context("Assets directory not found")?;

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
                file.read_to_string(&mut buf)
                    .with_context(|| "Failed to read assetidx")?;
                if buf == expected_file_contents {
                    continue;
                }
            }
            eprintln!("Updating assetidx: {:?}", asset_list_path);
            let mut asset_list_file =
                File::create(&asset_list_path).with_context(|| "Failed to create assetidx")?;

            asset_list_file
                .write_all(expected_file_contents.as_bytes())
                .with_context(|| "Failed to write to assetidx")?;
        }
    }
    Ok(())
}
