//! Provides an asset loader (`TmxMapLoader`) for Tiled Map Editor's `.tmx` files.
//! It includes a naive parser (`naive_tmx_loader`) to quickly extract top-level
//! map properties without full XML DOM parsing, optimizing initial load times when
//! inspecting multiple map files. The loaded asset (`TmxMap`) stores both the raw
//! TMX bytes for full parsing later and the naively extracted properties.
use bevy::{asset::AssetLoader, prelude::*, utils::HashMap};
use std::io::{BufRead, Cursor};
use thiserror::Error;

/// Holds a collection of naively parsed top-level properties from a TMX map file.
/// These properties are extracted quickly without full XML parsing for performance.
#[derive(Debug, Clone, Default)]
pub struct NaivelyParsedProps {
    /// Indicates if this map is intended as a campaign mission. Parsed from the 'is_campaign_mission' TMX property.
    pub is_campaign_mission: bool,
    /// The human-readable display name of the map. Parsed from the 'display_name' TMX property. Defaults to empty string.
    pub display_name: String,
    /// Descriptive or flavor text for the map/mission. Parsed from the 'flavor_text' TMX property.
    pub flavor_text: String,
    /// String defining the mission's order in a campaign sequence. Parsed from the 'campaign_order' TMX property.
    pub campaign_order: String,
    /// String representation of the difficulty for a campaign mission. Parsed from the 'campaign_difficulty' TMX property.
    pub campaign_difficulty_str: String,
    /// Filesystem path to a preview image for the map. Parsed from the 'map_preview_image' TMX property.
    pub map_preview_image: String,
    /// In-universe name of the location depicted by the map. Parsed from the 'location_name' TMX property.
    pub location_name: String,
    /// In-universe address of the location. Parsed from the 'location_address' TMX property.
    pub location_address: String,
}

/// Represents a Tiled map asset (`.tmx` file).
/// It stores the raw map data as bytes for deferred full parsing and a collection of
/// naively parsed top-level properties for quick access to essential metadata.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct TmxMap {
    /// Raw byte data of the TMX file, kept for full parsing when the map is actually loaded.
    pub bytes: Vec<u8>,
    /// The 'class' attribute of the `<map>` tag, naively parsed. Defaults to empty string if not found.
    pub class: String,
    /// Collection of naively parsed properties from the TMX file's top-level `<properties>` section.
    pub props: NaivelyParsedProps,
}

impl TmxMap {
    /// Creates a `TmxMap` instance from raw byte data.
    /// It uses `naive_tmx_loader` to quickly parse top-level properties
    /// and stores them along with the original bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, TmxMapError> {
        let props_map = naive_tmx_loader(Cursor::new(&bytes))?;

        let class = props_map.get("$class$").cloned().unwrap_or_default();

        // display_name is now part of NaivelyParsedProps, populated from the map or defaulted.
        let display_name_from_map = props_map.get("display_name").cloned().unwrap_or_default();

        let is_campaign_mission = props_map
            .get("is_campaign_mission")
            .is_some_and(|s| s == "true");

        // Populate NaivelyParsedProps, including display_name
        let parsed_props = NaivelyParsedProps {
            is_campaign_mission,
            display_name: display_name_from_map, // Use the value extracted from props_map
            flavor_text: props_map.get("flavor_text").cloned().unwrap_or_default(),
            campaign_order: props_map.get("campaign_order").cloned().unwrap_or_default(),
            campaign_difficulty_str: props_map
                .get("campaign_difficulty")
                .cloned()
                .unwrap_or_default(),
            map_preview_image: props_map
                .get("map_preview_image")
                .cloned()
                .unwrap_or_default(),
            location_name: props_map.get("location_name").cloned().unwrap_or_default(),
            location_address: props_map
                .get("location_address")
                .cloned()
                .unwrap_or_default(),
        };

        Ok(Self {
            bytes,
            class,
            props: parsed_props, // Store the populated NaivelyParsedProps
        })
    }
}

/// Errors that can occur during the loading of a `TmxMap` asset.
#[derive(Error, Debug)]
pub enum TmxMapError {
    /// An I/O error occurred while reading the map file.
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// An unknown error occurred.
    #[error("unknown TmxMap load error")]
    Unknown,
}

/// Bevy `AssetLoader` for `TmxMap` assets.
/// It reads the raw bytes of a `.tmx` file and uses `TmxMap::from_bytes`
/// for initial naive property parsing.
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

/// **Warning:** This function performs a naive, line-by-line parse of the TMX file
/// to quickly extract top-level map properties without incurring the cost of a full
/// XML DOM parsing and tileset resolution. This is crucial for performance when
/// needing to inspect many map files at startup (e.g., for listing available campaign missions).
/// Attempting to replace this with a full `tiled::Loader` parse for this specific purpose
/// would significantly slow down initial game load times if many maps are present.
///
/// It returns a HashMap of property names to their string values.
/// The 'class' attribute from the `<map>` tag is stored with a special key `"$class$"`.
pub fn naive_tmx_loader(reader: impl BufRead) -> std::io::Result<HashMap<String, String>> {
    let mut properties_map = HashMap::new();
    let mut in_map_properties_section = false;

    for line_result in reader.lines() {
        let line = line_result?.trim().to_owned();

        if line.starts_with("<map") {
            const CLASS_MARKER: &str = " class=\"";
            if let Some(start_idx) = line.find(CLASS_MARKER) {
                let remainder = &line[start_idx + CLASS_MARKER.len()..];
                if let Some(end_idx) = remainder.find('"') {
                    properties_map.insert(
                        "$class$".to_string(),
                        xml_unescape(remainder[..end_idx].to_string()),
                    );
                }
            }
        } else if line.starts_with("<properties>") {
            in_map_properties_section = true;
        } else if line.starts_with("</properties>") {
            break;
        } else if in_map_properties_section && line.starts_with("<property name=") {
            let get_attribute_value = |prop_line: &str, attr_name: &str| -> Option<String> {
                let marker = format!("{}=\"", attr_name);
                if let Some(start) = prop_line.find(&marker) {
                    let remainder = &prop_line[start + marker.len()..];
                    if let Some(end) = remainder.find('"') {
                        return Some(xml_unescape(remainder[..end].to_string()));
                    }
                }
                None
            };

            if let Some(name) = get_attribute_value(&line, "name") {
                if let Some(value) = get_attribute_value(&line, "value") {
                    properties_map.insert(name, value);
                } else {
                    properties_map.insert(name, "".to_string());
                }
            }
        }
    }
    Ok(properties_map)
}

/// Undoes basic XML escaping for attribute values.
/// Replaces common XML entities like `&`, `<`, etc., with their
/// corresponding characters.
fn xml_unescape(val: String) -> String {
    val.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}
