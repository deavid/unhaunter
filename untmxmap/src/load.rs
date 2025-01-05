use bevy::utils::HashMap;
use uncore::types::tiledmap::map::{
    MapLayer, MapLayerGroup, MapLayerType, MapTile, MapTileList, Pos,
};

// ----------- Load functions -------------------
/// Entry point for loading tiled maps.
///
/// Example: let mut loader = tiled::Loader::new(); let map =
/// loader.load_tmx_map("assets/maps/map_house1_3x.tmx").unwrap(); let map_layers =
/// load_tile_layer_iter(map.layers());
pub fn load_tile_layer_iter<'a>(
    layer_iter: impl ExactSizeIterator<Item = tiled::Layer<'a>>,
) -> Vec<MapLayer> {
    let mut ret = vec![];
    for layer in layer_iter {
        let mut user_properties = HashMap::new();
        for (k, v) in &layer.properties {
            user_properties.insert(k.clone(), v.clone());
        }
        let map_layer = MapLayer {
            name: layer.name.to_string(),
            visible: layer.visible,
            offset: Pos::new(layer.offset_x, layer.offset_y),
            user_class: layer.user_type.clone(),
            user_properties,
            opacity: layer.opacity,
            data: load_tile_layer(layer),
        };
        ret.push(map_layer);
    }
    ret
}

fn load_tile_layer(layer: tiled::Layer) -> MapLayerType {
    match layer.layer_type() {
        tiled::LayerType::Tiles(tile_layer) => {
            MapLayerType::Tiles(load_tile_layer_tiles(tile_layer))
        }
        tiled::LayerType::Objects(_) => MapLayerType::Objects(),
        tiled::LayerType::Image(_) => MapLayerType::Image(),
        tiled::LayerType::Group(grp_layer) => MapLayerType::Group(load_tile_group_layer(grp_layer)),
    }
}

fn load_tile_group_layer(layer: tiled::GroupLayer) -> MapLayerGroup {
    let layers = load_tile_layer_iter(layer.layers());
    MapLayerGroup { layers }
}

fn load_tile_layer_tiles(layer: tiled::TileLayer) -> MapTileList {
    let mut ret = vec![];
    for y in 0..layer.height().unwrap() as i32 {
        for x in 0..layer.width().unwrap() as i32 {
            let maybe_tile = layer.get_tile(x, y);
            if let Some(tile) = maybe_tile {
                let t = MapTile {
                    pos: Pos::new(x, y),
                    tileset: tile.get_tileset().name.to_string(),
                    tileuid: tile.id(),
                    flip_x: tile.flip_h,
                };
                ret.push(t);
            }
        }
    }
    MapTileList { v: ret }
}
