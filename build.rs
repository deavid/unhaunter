use std::fs::File;
use std::io::Write;
use walkdir::WalkDir;

fn get_asset_types() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("fonts", vec!["ttf"]),
        ("img", vec!["png"]),
        ("maps", vec!["tmx", "tsx"]),
        ("music", vec!["ogg"]),
        ("sounds", vec!["ogg"]),
        ("manual", vec!["png"]),
        ("phrasebooks", vec!["yaml"]),
    ]
}

fn get_asset_list() -> Vec<String> {
    let mut list = vec![];
    let assets_dir = "assets/";
    for entry in WalkDir::new(assets_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let relative_path = path.strip_prefix(assets_dir).unwrap();
            list.push(relative_path.to_string_lossy().to_string());
        }
    }
    list
}

fn main() {
    let asset_list = get_asset_list();
    let asset_types = get_asset_types();

    for (folder_name, ext_list) in asset_types {
        for ext in &ext_list {
            let asset_list_path = format!("assets/index/{folder_name}-{ext}.assetidx");

            let mut asset_list_file =
                File::create(asset_list_path).expect("Failed to create assetidx");

            for path in &asset_list {
                if path.starts_with(folder_name) && path.ends_with(ext) {
                    writeln!(asset_list_file, "{}", path).expect("Failed to write to assetidx");
                }
            }
        }
    }
}
