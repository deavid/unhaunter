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
    pub fn resolve(&self, original_path: &str, max_factor: u32) -> Option<ResolvedUpscale> {
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
            (asset_server.load(path.to_string()), 1.0)
        }
    }
}
