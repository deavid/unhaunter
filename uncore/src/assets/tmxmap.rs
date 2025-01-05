use bevy::{asset::AssetLoader, prelude::*};
use std::io::BufRead;
use thiserror::Error;

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

/// Loads a TMX as text file and inspects the first lines to obtain class and
/// display_name.
pub fn naive_tmx_loader(reader: impl BufRead) -> std::io::Result<(Option<String>, Option<String>)> {
    let mut class: Option<String> = None;
    let mut display_name: Option<String> = None;
    for line in reader.lines().take(10) {
        let line = line?.trim().to_owned();
        if line.starts_with("<map") {
            const CLASS_STR: &str = " class=\"";
            if let Some(classpos) = line.find(CLASS_STR) {
                let p_line = &line[classpos + CLASS_STR.len()..];
                if let Some(rpos) = p_line.find('"') {
                    class = Some(p_line[..rpos].to_string());
                }
            }
        }
        if line.starts_with("<property name=\"display_name\"") {
            const VALUE_STR: &str = " value=\"";
            if let Some(valpos) = line.find(VALUE_STR) {
                let p_line = &line[valpos + VALUE_STR.len()..];
                if let Some(rpos) = p_line.find('"') {
                    display_name = Some(p_line[..rpos].to_string());
                }
            }
        }
    }
    Ok((class, display_name))
}
