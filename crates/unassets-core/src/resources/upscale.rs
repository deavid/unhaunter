use bevy::prelude::*;
use std::collections::HashMap;

/// Result of a resolution lookup
#[derive(Debug, Clone)]
pub struct ResolvedUpscale {
    pub path: String,
    pub factor: f32,
}

/// Resource that tracks available upscaled assets
#[derive(Resource, Default, Debug)]
pub struct UpscaleIndex {
    /// Maps original path (relative to assets/) to available upscaled versions
    /// e.g. "img/items.png" -> { 3: "upscaled/zoom03x_items.png" }
    pub available: HashMap<String, HashMap<u32, String>>,
}

impl UpscaleIndex {
    pub fn resolve(&self, path: &str, max_factor: u32) -> Option<ResolvedUpscale> {
        // If max_factor is 1, we always want the original, no matter what.
        if max_factor <= 1 {
            return None;
        }

        let path = path.strip_prefix("assets/").unwrap_or(path);

        // Try to see if the path is already an upscaled one
        let (original_path, _current_factor) =
            if let Some(rest) = path.strip_prefix("upscaled/zoom0") {
                if rest.get(1..3).map(|s| s == "x_").unwrap_or(false) {
                    let factor_char = rest.chars().next().unwrap();
                    let factor = factor_char.to_digit(10).unwrap_or(1);
                    (&rest[3..], factor)
                } else {
                    (path, 1)
                }
            } else {
                (path, 1)
            };

        let available_factors = self.available.get(original_path)?;

        for factor in (2..=max_factor).rev() {
            if let Some(upscaled_path) = available_factors.get(&factor) {
                return Some(ResolvedUpscale {
                    path: upscaled_path.clone(),
                    factor: factor as f32,
                });
            }
        }
        None
    }

    pub fn load_upscaled(
        &self,
        path: &str,
        asset_server: &AssetServer,
        max_factor: u32,
    ) -> (Handle<Image>, f32) {
        if let Some(resolved) = self.resolve(path, max_factor) {
            (asset_server.load(resolved.path), resolved.factor)
        } else {
            // Even if resolve returns None, we should try to detect if we're already upscaled
            let mut factor = 1.0;
            let check_path = path.strip_prefix("assets/").unwrap_or(path);
            if let Some(rest) = check_path.strip_prefix("upscaled/zoom0")
                && rest.get(1..3).map(|s| s == "x_").unwrap_or(false)
            {
                let factor_char = rest.chars().next().unwrap();
                factor = factor_char.to_digit(10).unwrap_or(1) as f32;
            }
            (asset_server.load(path.to_string()), factor)
        }
    }
}
