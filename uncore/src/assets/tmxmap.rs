//! Provides an asset loader (`TmxMapLoader`) for Tiled Map Editor's `.tmx` files.
//! It includes a naive parser (`naive_tmx_loader`) to quickly extract top-level
//! map properties without full XML DOM parsing, optimizing initial load times when
//! inspecting multiple map files. The loaded asset (`TmxMap`) stores both the raw
//! TMX bytes for full parsing later and the naively extracted properties.
use bevy::{asset::AssetLoader, prelude::*};
use bevy_platform::collections::HashMap;
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

    /// The base reward for completing the mission.
    pub mission_reward_base: i64,

    /// The required deposit for the mission.
    pub required_deposit: i64,

    /// Score threshold for achieving grade A
    pub grade_a_score_threshold: i64,

    /// Score threshold for achieving grade B
    pub grade_b_score_threshold: i64,

    /// Score threshold for achieving grade C
    pub grade_c_score_threshold: i64,

    /// Score threshold for achieving grade D
    pub grade_d_score_threshold: i64,

    /// Minimum player level required for this map. Parsed from the 'min_player_level' TMX property. Defaults to 0.
    pub min_player_level: i32,
    /// Indicates if this map is a draft and should be ignored by the game. Parsed from the 'draft' TMX property.
    pub draft: bool,
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
        let display_name_from_map = props_map.get("display_name").cloned().unwrap_or_default();
        let is_campaign_mission = props_map
            .get("is_campaign_mission")
            .is_some_and(|s| s == "true");
        let draft = props_map.get("draft").is_some_and(|s| s == "true"); // Added this line

        let parse_i64 = |key: &str, default: i64| -> i64 {
            props_map
                .get(key)
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(default)
        };
        let parse_i32 = |key: &str, default: i32| -> i32 {
            // Helper for i32
            props_map
                .get(key)
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(default)
        };

        let mut parsed_props = NaivelyParsedProps {
            is_campaign_mission,
            display_name: display_name_from_map,
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
            mission_reward_base: parse_i64("mission_reward_base", 0),
            required_deposit: parse_i64("required_deposit", 0),
            // FIXME: The default values for these properties should be computed by `calculate_grade_thresholds`.
            // This is a temporary fix to ensure they are initialized.
            grade_a_score_threshold: parse_i64("grade_a_score_threshold", 1000),
            grade_b_score_threshold: parse_i64("grade_b_score_threshold", 500),
            grade_c_score_threshold: parse_i64("grade_c_score_threshold", 250),
            grade_d_score_threshold: parse_i64("grade_d_score_threshold", 125),
            min_player_level: parse_i32("min_player_level", 0), // Parse min_player_level
            draft,                                              // Added this line
        };

        // Ensure grade thresholds are fully initialized
        let (grade_a, grade_b, grade_c, grade_d) = Self::calculate_grade_thresholds(&parsed_props);
        parsed_props.grade_a_score_threshold = grade_a;
        parsed_props.grade_b_score_threshold = grade_b;
        parsed_props.grade_c_score_threshold = grade_c;
        parsed_props.grade_d_score_threshold = grade_d;

        Ok(Self {
            bytes,
            class,
            props: parsed_props,
        })
    }

    fn calculate_grade_thresholds(props: &NaivelyParsedProps) -> (i64, i64, i64, i64) {
        const GOLDEN_RATIO: f32 = 1.618;
        const DEFAULT_A: i64 = 1000;
        const DEFAULT_B: i64 = 500;
        const DEFAULT_C: i64 = 250;
        const DEFAULT_D: i64 = 125;

        let mut grade_a = Some(props.grade_a_score_threshold);
        let mut grade_b = Some(props.grade_b_score_threshold);
        let mut grade_c = Some(props.grade_c_score_threshold);
        let mut grade_d = Some(props.grade_d_score_threshold);

        if grade_a.is_none() && grade_b.is_none() && grade_c.is_none() && grade_d.is_none() {
            grade_a = Some(DEFAULT_A);
            grade_b = Some(DEFAULT_B);
            grade_c = Some(DEFAULT_C);
            grade_d = Some(DEFAULT_D);
        } else if let Some(a_val) = grade_a {
            if grade_b.is_none() {
                grade_b = Some((a_val as f32 / GOLDEN_RATIO).round() as i64);
            }
            if let Some(b_val) = grade_b {
                if grade_c.is_none() {
                    grade_c = Some((b_val as f32 / GOLDEN_RATIO).round() as i64);
                }
            }
            if let Some(c_val) = grade_c {
                if grade_d.is_none() {
                    grade_d = Some((c_val as f32 / GOLDEN_RATIO).round() as i64);
                }
            }
        } else if let Some(d_val) = grade_d {
            if grade_c.is_none() {
                grade_c = Some((d_val as f32 * GOLDEN_RATIO).round() as i64);
            }
            if let Some(c_val) = grade_c {
                if grade_b.is_none() {
                    grade_b = Some((c_val as f32 * GOLDEN_RATIO).round() as i64);
                }
            }
            if let Some(b_val) = grade_b {
                if grade_a.is_none() {
                    grade_a = Some((b_val as f32 * GOLDEN_RATIO).round() as i64);
                }
            }
        }

        (
            grade_a.unwrap_or(DEFAULT_A),
            grade_b.unwrap_or(DEFAULT_B),
            grade_c.unwrap_or(DEFAULT_C),
            grade_d.unwrap_or(DEFAULT_D),
        )
    }

    /// Parse an i64 value from a string property.
    /// Returns the default value if the string is empty or cannot be parsed.
    pub fn parse_i64_property(
        &self,
        val_str: Option<&str>,
        property_name: &str,
        default_value: i64,
    ) -> i64 {
        let Some(val_str) = val_str else {
            return default_value;
        };

        if val_str.is_empty() {
            return default_value;
        }

        val_str.parse::<i64>().unwrap_or_else(|e| {
            warn!(
                "TMX property '{}' ('{}') parse error: {:?}. Defaulting to {}.",
                property_name, val_str, e, default_value
            );
            default_value
        })
    }

    /// Get the mission reward base value, parsed to i64
    pub fn mission_reward_base(&self) -> i64 {
        self.props.mission_reward_base
    }

    /// Get the required deposit value, parsed to i64
    pub fn required_deposit(&self) -> i64 {
        self.props.required_deposit
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
        } else if in_map_properties_section && line.starts_with("<property name=\"") {
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
