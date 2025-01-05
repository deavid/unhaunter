use crate::naive::naive_tmx_loader;
use bevy::prelude::*;
use glob::Pattern;
use uncore::{resources::maps::Maps, types::root::map::Map};
use walkdir::WalkDir;

pub fn init_maps(maps: ResMut<Maps>) {
    arch::init_maps(maps)
}

#[cfg(not(target_arch = "wasm32"))]
mod arch {
    use super::*;

    /// Scans the "assets/maps/" directory for files matching "*.tmx" and returns their
    /// paths.
    fn find_tmx_files() -> Vec<String> {
        let mut paths = Vec::new();
        let pattern = Pattern::new("*.tmx").unwrap();
        let base_path = "assets/maps/";
        info!("Loading maps...");
        for entry in WalkDir::new(base_path).into_iter() {
            let Ok(entry) = entry else {
                error!("Error loading: {:?}", entry);
                continue;
            };
            let path = entry.path();
            info!("Found {:?}", path);

            // Check if the path matches the "*.tmx" pattern and is a file
            if path.is_file() && pattern.matches_path(path) {
                // Convert the path to a String and store it in the vector
                if let Some(str_path) = path.to_str() {
                    paths.push(str_path.to_string());
                }
            }
        }
        paths.sort();
        paths
    }

    pub fn init_maps(mut maps: ResMut<Maps>) {
        // Scan for maps:
        let tmx_files = find_tmx_files();
        for path in tmx_files {
            // Loading a map can take 100ms or more. Therefore we do a naive load instead
            let (classname, display_name) = match naive_tmx_loader(&path) {
                Ok(m) => m,
                Err(e) => {
                    warn!("Cannot load map {path:?}: {e}");
                    continue;
                }
            };
            if classname != Some("UnhaunterMap1".to_string()) {
                warn!(
                    "Unrecognized Class {:?} for map {:?} (Should be 'UnhaunterMap1')",
                    classname, path
                );
                continue;
            }
            let default_name = format!("Unnamed ({})", path.replace("assets/maps/", ""));
            let display_name = display_name.unwrap_or(default_name);
            info!("Found map {display_name:?} at path {path:?}");
            maps.maps.push(Map {
                name: display_name,
                path,
            });
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod arch {
    use super::*;

    fn find_tmx_files() -> Vec<(String, String)> {
        // WASM does not support scanning folders it seems...
        vec![
            (
                "assets/maps/map_house1.tmx".to_string(),
                "123 Acorn Lane Street House".to_string(),
            ),
            (
                "assets/maps/map_house2.tmx".to_string(),
                "4567 Chocolate Boulevard Street House".to_string(),
            ),
            (
                "assets/maps/map_school1.tmx".to_string(),
                "99 Unicorn Way University".to_string(),
            ),
            (
                "assets/maps/tut01_basics.tmx".to_string(),
                "Tutorial 01: Basics".to_string(),
            ),
            (
                "assets/maps/tut02_glass_house.tmx".to_string(),
                "Tutorial 02: Glass House".to_string(),
            ),
        ]
    }

    pub fn init_maps(mut maps: ResMut<Maps>) {
        // Scan for maps:
        let tmx_files = find_tmx_files();
        for (path, display_name) in tmx_files {
            // let display_name = path.replace("assets/maps/", "");
            info!("Found map {display_name:?} at path {path:?}");
            maps.maps.push(Map {
                name: display_name,
                path,
            });
        }
    }
}
