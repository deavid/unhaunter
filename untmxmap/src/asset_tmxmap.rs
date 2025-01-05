use bevy::{asset::AssetLoader, prelude::*};
use thiserror::Error;

use crate::naive::naive_tmx_loader;

#[derive(Asset, Reflect)]
pub struct TmxMap {
    pub bytes: Vec<u8>,
    pub class: Option<String>,
    pub display_name: Option<String>,
}

impl TmxMap {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, TmxMapError> {
        let (class, display_name) = naive_tmx_loader(&bytes[..])?;

        Ok(Self {
            bytes,
            class,
            display_name,
        })
    }
}

#[derive(Error, Debug)]
pub enum TmxMapError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("unknown TmxMap load error")]
    Unknown,
}

#[derive(Default)]
pub struct TmxMapLoader;

impl AssetLoader for TmxMapLoader {
    type Asset = TmxMap;

    type Settings = ();

    type Error = TmxMapError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        TmxMap::from_bytes(bytes)
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
}
