use bevy::prelude::*;
use uncore::assets::index::AssetIdx;
use uncore::assets::tmxmap::TmxMap;
use uncore::assets::tsxsheet::TsxSheet;
use uncore::difficulty::Difficulty;
use uncore::resources::cli_options::CliOptions;
use uncore::types::mission_data::MissionData;
use uncore::types::root::map::Sheet;
use uncore::{resources::maps::Maps, types::root::map::Map};

pub struct PreLoad<A: Asset> {
    path: String,
    handle: Handle<A>,
    processed: bool,
}

#[derive(Resource, Default)]
pub struct MapAssetIndexHandle {
    tmxidx: Handle<AssetIdx>,
    tsxidx: Handle<AssetIdx>,
    idxprocessed: bool,
    maps: Vec<PreLoad<TmxMap>>,
    sheets: Vec<PreLoad<TsxSheet>>,
}

impl MapAssetIndexHandle {
    /// Returns true if all maps and sheets have been processed
    pub fn all_processed(&self) -> bool {
        if !self.idxprocessed {
            warn!(
                "Index not processed: maps:{:?} processed:{}",
                self.maps.len(),
                self.idxprocessed
            );
            return false;
        }

        // If there are any unprocessed maps, return false
        if let Some(map) = self.maps.iter().find(|map| !map.processed) {
            warn!("Map {} is not processed", map.path);
            return false;
        }

        // Check if all sheets are processed
        if let Some(sheet) = self.sheets.iter().find(|sheet| !sheet.processed) {
            warn!("Sheet {} is not processed", sheet.path);
            return false;
        }

        true
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Startup, init_maps);
    app.add_systems(Update, (map_index_preload, tmxmap_preload));
}

fn init_maps(asset_server: Res<AssetServer>, mut mapsidx: ResMut<MapAssetIndexHandle>) {
    mapsidx.tmxidx = asset_server.load("index/maps-tmx.assetidx");
    mapsidx.tsxidx = asset_server.load("index/maps-tsx.assetidx");
}

fn map_index_preload(
    asset_server: Res<AssetServer>,
    idx_assets: Res<Assets<AssetIdx>>,
    mut mapsidx: ResMut<MapAssetIndexHandle>,
) {
    if mapsidx.idxprocessed {
        return;
    }
    let Some(maps) = idx_assets.get(&mapsidx.tmxidx) else {
        return;
    };
    let Some(sheets) = idx_assets.get(&mapsidx.tsxidx) else {
        return;
    };
    for path in &maps.assets {
        let handle: Handle<TmxMap> = asset_server.load(path);
        let path = path.to_string();
        mapsidx.maps.push(PreLoad {
            handle,
            path,
            processed: false,
        });
    }
    for path in &sheets.assets {
        let handle: Handle<TsxSheet> = asset_server.load(path);
        let path = path.to_string();
        mapsidx.sheets.push(PreLoad {
            handle,
            path,
            processed: false,
        });
    }
    mapsidx.idxprocessed = true;
}

fn tmxmap_preload(
    mut maps: ResMut<Maps>,
    tmx_assets: Res<Assets<TmxMap>>,
    mut mapsidx: ResMut<MapAssetIndexHandle>,
    cli_options: Res<CliOptions>,
) {
    let mut cleanup_needed = false;
    if !mapsidx.idxprocessed {
        return;
    }
    for mapload in &mut mapsidx.maps {
        if mapload.processed {
            continue;
        }
        let tmx = tmx_assets.get(&mapload.handle);
        if let Some(tmx) = tmx {
            mapload.processed = true;
            cleanup_needed = true;

            // If the map is a draft, skip loading it unless --draft-maps is passed.
            if tmx.props.draft && !cli_options.include_draft_maps {
                warn!(
                    "Skipping draft map {:?} at path {:?} (use --draft-maps to include)",
                    tmx.props.display_name, mapload.path
                );
                continue;
            }

            let path = mapload.path.clone();
            let classname = tmx.class.clone();
            let display_name = tmx.props.display_name.clone();

            if classname != "UnhaunterMap1" {
                warn!(
                    "Unrecognized Class {:?} for map {:?} (Should be 'UnhaunterMap1')",
                    classname, path
                );
                continue;
            }

            let default_name = format!("Unnamed ({})", path.replace("maps/", ""));
            let display_name = if display_name.is_empty() {
                default_name
            } else {
                display_name
            };
            debug!("Found map {display_name:?} at path {path:?}");

            // Create mission_data if the map has relevant properties
            let mission_data = create_mission_data(tmx, &path);

            maps.maps.push(Map {
                name: display_name,
                path,
                handle: mapload.handle.clone(),
                mission_data,
            });
        }
    }
    maps.maps.sort_by_key(|x| x.path.clone());
    for sheet in &mut mapsidx.sheets {
        if !sheet.processed {
            maps.sheets.push(Sheet {
                path: sheet.path.clone(),
                handle: sheet.handle.clone(),
            });
            sheet.processed = true;
            cleanup_needed = true;
        }
    }
    if cleanup_needed {
        mapsidx.maps.retain(|x| !x.processed);
        mapsidx.sheets.retain(|x| !x.processed);
    }
}

/// Create a MissionData instance from a TmxMap's properties if it's a valid mission
fn create_mission_data(tmx: &TmxMap, path: &str) -> MissionData {
    let props = &tmx.props;

    // Determine if this is a campaign mission
    let is_campaign_mission = props.is_campaign_mission;

    // Parse difficulty - only needed for campaign missions but we'll set a default
    // for custom missions too
    let difficulty_enum = if !props.campaign_difficulty_str.is_empty() {
        match props.campaign_difficulty_str.as_str() {
            "TutorialChapter1" => Difficulty::TutorialChapter1,
            "TutorialChapter2" => Difficulty::TutorialChapter2,
            "TutorialChapter3" => Difficulty::TutorialChapter3,
            "TutorialChapter4" => Difficulty::TutorialChapter4,
            "TutorialChapter5" => Difficulty::TutorialChapter5,
            "StandardChallenge" => Difficulty::StandardChallenge,
            "HardChallenge" => Difficulty::HardChallenge,
            "ExpertChallenge" => Difficulty::ExpertChallenge,
            "MasterChallenge" => Difficulty::MasterChallenge,
            _ => {
                warn!(
                    "Unknown campaign_difficulty string '{}' in map '{}'. Defaulting to StandardChallenge.",
                    props.campaign_difficulty_str, path
                );
                Difficulty::StandardChallenge // Sensible default
            }
        }
    } else {
        Difficulty::StandardChallenge // Default difficulty for maps without specifics
    };

    MissionData {
        id: path.to_string(),
        map_filepath: path.to_string(),
        display_name: props.display_name.clone(),
        flavor_text: props.flavor_text.clone(),
        order: props.campaign_order.clone(),
        difficulty: difficulty_enum,
        is_campaign_mission,
        preview_image_path: props.map_preview_image.clone(),
        location_name: props.location_name.clone(),
        location_address: props.location_address.clone(),
        mission_reward_base: props.mission_reward_base,
        required_deposit: props.required_deposit,
        grade_a_score_threshold: props.grade_a_score_threshold,
        grade_b_score_threshold: props.grade_b_score_threshold,
        grade_c_score_threshold: props.grade_c_score_threshold,
        grade_d_score_threshold: props.grade_d_score_threshold,
        min_player_level: props.min_player_level,
    }
}
