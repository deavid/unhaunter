use bevy::prelude::*;
// No HashSet or AssetId needed if the system runs once reliably after all assets are loaded.

// Use existing uncore types
use uncore::{
    assets::tmxmap::TmxMap, // TmxMap now contains NaivelyParsedProps
    difficulty::Difficulty,
    resources::maps::Maps, // Your existing Maps resource holding Vec<uncore::types::root::map::Map>
    types::campaign::{CampaignMissionData, CampaignMissionsResource},
};

/// Bevy system to parse TMX assets and populate the `CampaignMissionsResource`.
///
/// This system should run once after all TMX map assets are loaded and the `Maps` resource
/// is populated. It iterates through known maps, checks their naively parsed properties,
/// and creates `CampaignMissionData` for those flagged as campaign missions.
pub fn load_campaign_missions_into_resource(
    tmx_assets: Res<Assets<TmxMap>>,
    maps_resource: Res<Maps>, // Your existing struct that lists all map paths and handles
    // campaign_missions_res_opt: Option<ResMut<CampaignMissionsResource>>, // Use Option for robust init
    // Let's make it simpler: if the resource doesn't exist, create it. If it does, clear and repopulate.
    // This requires the system to be infallibly run only once, or manage state with a Local.
    // For simplicity with a one-shot system (e.g. OnEnter(AppState::MainMenu)), we can do this:
    // Alternatively, query for the resource and only proceed if it's not yet populated,
    // or clear it if it is. Let's assume ResMut for now and the system is guarded.
    mut campaign_missions_res: ResMut<CampaignMissionsResource>,
    mut processed_flag: Local<bool>, // Ensures this system effectively runs once.
) {
    // Only consider the system fully processed if we actually found maps
    if *processed_flag && !maps_resource.maps.is_empty() {
        return;
    }

    // Reset the processed flag if no maps were loaded yet
    if maps_resource.maps.is_empty() {
        return;
    }

    info!("[CAMPAIGN DEBUG] Starting campaign mission loading process...");
    info!(
        "[CAMPAIGN DEBUG] Total maps in maps_resource: {}",
        maps_resource.maps.len()
    );

    let mut collected_missions: Vec<CampaignMissionData> = Vec::new();
    let mut all_tmx_assets_available = true;

    // Iterate through your `maps_resource.maps` which contains `uncore::types::root::map::Map`
    for map_entry in maps_resource.maps.iter() {
        // map_entry is of type uncore::types::root::map::Map
        // map_entry.path is the path string
        // map_entry.handle is the Handle<TmxMap>

        info!("[CAMPAIGN DEBUG] Checking map: {}", map_entry.path);

        if let Some(tmx_asset) = tmx_assets.get(&map_entry.handle) {
            // Access the naively parsed properties directly from the TmxMap asset
            let props = &tmx_asset.props;

            info!(
                "[CAMPAIGN DEBUG] Map properties - is_campaign_mission: {}, display_name: {}, campaign_order: {}, campaign_difficulty: {}",
                props.is_campaign_mission,
                props.display_name,
                props.campaign_order,
                props.campaign_difficulty_str
            );

            if props.is_campaign_mission {
                let difficulty_enum = match props.campaign_difficulty_str.as_str() {
                    "TutorialChapter1" => Difficulty::TutorialChapter1,
                    "TutorialChapter2" => Difficulty::TutorialChapter2,
                    "TutorialChapter3" => Difficulty::TutorialChapter3,
                    "TutorialChapter4" => Difficulty::TutorialChapter4,
                    "TutorialChapter5" => Difficulty::TutorialChapter5,
                    "StandardChallenge" => Difficulty::StandardChallenge,
                    "HardChallenge" => Difficulty::HardChallenge,
                    "ExpertChallenge" => Difficulty::ExpertChallenge,
                    "MasterChallenge" => Difficulty::MasterChallenge,
                    // Add any other consolidated difficulty names here
                    _ => {
                        warn!(
                            "Unknown campaign_difficulty string '{}' in map '{}'. Defaulting to TutorialChapter1.",
                            props.campaign_difficulty_str, map_entry.path
                        );
                        Difficulty::TutorialChapter1 // Sensible default
                    }
                };

                info!(
                    "[CAMPAIGN DEBUG] Adding map as campaign mission: {}",
                    props.display_name
                );

                collected_missions.push(CampaignMissionData {
                    id: map_entry.path.clone(), // Use map path as a unique ID
                    map_filepath: map_entry.path.clone(),
                    display_name: props.display_name.clone(),
                    flavor_text: props.flavor_text.clone(),
                    order: props.campaign_order.clone(),
                    difficulty: difficulty_enum,
                    preview_image_path: props.map_preview_image.clone(),
                    location_name: props.location_name.clone(),
                    location_address: props.location_address.clone(),
                });
            } else {
                info!(
                    "[CAMPAIGN DEBUG] Skipping map, not a campaign mission: {}",
                    map_entry.path
                );
            }
        } else {
            // This case means the TmxMap asset for a path listed in `maps_resource.maps`
            // is not yet fully loaded into `Assets<TmxMap>`.
            // This system should be scheduled to run after all TMX assets are loaded.
            warn!(
                "TmxMap asset for map path '{}' not yet available in Assets<TmxMap>. Deferring campaign mission loading.",
                map_entry.path
            );
            all_tmx_assets_available = false;
            break; // Stop processing and wait for assets to load.
        }
    }

    if !all_tmx_assets_available {
        return; // System will run again, hopefully assets are loaded then.
    }

    // Sort the collected missions by their 'order' field
    collected_missions.sort_by(|a, b| a.order.cmp(&b.order));

    // Populate the resource
    // This overwrites any previous data, suitable for a one-time population.
    campaign_missions_res.missions = collected_missions;

    info!(
        "CampaignMissionsResource populated and sorted with {} missions.", // Updated log
        campaign_missions_res.missions.len()
    );

    // Log the order of missions for verification
    if campaign_missions_res.missions.is_empty() {
        warn!("[CAMPAIGN DEBUG] ⚠️ No campaign missions found! Check map properties.");
    } else {
        info!("[CAMPAIGN DEBUG] Campaign missions found:");
        for (idx, mission) in campaign_missions_res.missions.iter().enumerate() {
            info!(
                "[CAMPAIGN DEBUG] Mission [{}]: Order '{}' - '{}' ({})",
                idx, mission.order, mission.display_name, mission.map_filepath
            );
        }
    }

    *processed_flag = true; // Mark as processed.
}
