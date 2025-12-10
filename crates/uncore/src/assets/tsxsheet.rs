use bevy::{asset::AssetLoader, prelude::*};

/// Loads a TSX Sheet using Bevy machinery - to avoid reading the files manually
/// so that this is compatible with WASM
#[derive(Asset, Reflect)]
pub struct TsxSheet {
    pub bytes: Vec<u8>,
}

impl TsxSheet {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

#[derive(Default)]
pub struct TsxSheetLoader;

impl AssetLoader for TsxSheetLoader {
    type Asset = TsxSheet;

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
        Ok(TsxSheet::from_bytes(bytes))
    }

    fn extensions(&self) -> &[&str] {
        &["tsx"]
    }
}
