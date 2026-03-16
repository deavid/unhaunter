use bevy::asset::AssetEvent;
use bevy::audio::AudioSource;
use bevy::prelude::*;
use unassets_core::assets::index::AssetIdx;
use unrender_std::materials::CustomMaterial1;
use untmxmap_core::assets::tmxmap::TmxMap;
use untmxmap_core::assets::tsxsheet::TsxSheet;

fn log_asset_events<T: Asset>(
    mut events: MessageReader<AssetEvent<T>>,
    asset_server: Res<AssetServer>,
) {
    if !cfg!(debug_assertions) {
        return;
    }

    let type_name = std::any::type_name::<T>();
    for event in events.read() {
        match event {
            AssetEvent::Added { id } => {
                let _path = asset_server
                    .get_path(*id)
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "<generated>".to_string());
                // trace!("asset loaded: type={type_name} path={path} id={id:?}");
            }
            AssetEvent::Removed { id } => {
                let path = asset_server
                    .get_path(*id)
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "<unknown>".to_string());
                trace!("asset unloaded: type={type_name} path={path} id={id:?}");
            }
            _ => {}
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    if !cfg!(debug_assertions) {
        return;
    }
    let is_headless = app
        .world()
        .get_resource::<untypes_core::cli::CliOptions>()
        .map(|cli| cli.dedicated)
        .unwrap_or(false);

    if is_headless {
        return;
    }

    app.add_systems(
        Update,
        (
            log_asset_events::<Image>,
            log_asset_events::<AudioSource>,
            log_asset_events::<Font>,
            log_asset_events::<TextureAtlasLayout>,
            log_asset_events::<Mesh>,
            log_asset_events::<CustomMaterial1>,
            log_asset_events::<AssetIdx>,
            log_asset_events::<TmxMap>,
            log_asset_events::<TsxSheet>,
        ),
    );
}
