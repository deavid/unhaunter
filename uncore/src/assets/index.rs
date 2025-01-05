use bevy::{asset::AssetLoader, prelude::*};

#[derive(Asset, Reflect)]
pub struct AssetIdx {
    pub assets: Vec<String>,
}

impl AssetIdx {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let data = String::from_utf8_lossy(&bytes);

        let assets: Vec<String> = data
            .split('\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self { assets }
    }
}

#[derive(Default)]
pub struct AssetIdxLoader;

impl AssetLoader for AssetIdxLoader {
    type Asset = AssetIdx;

    type Settings = ();

    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(AssetIdx::from_bytes(bytes))
    }

    fn extensions(&self) -> &[&str] {
        &["assetidx"]
    }
}
