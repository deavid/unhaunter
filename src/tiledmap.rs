//! Functionality to load Tiled maps (tilesets and tilemaps) into Bevy for Unhaunter.
//!
//! Most of the classes here are almost a redefinition (for now) of the tiled library.
//! Currently serve as an example on how to load/store data.

use std::{collections::HashMap, fmt::Debug, slice::Iter};

/// A simple 2D position with X and Y components that it is generic.
///
/// This is mainly used to customize the Debug output so it is shorter.
#[derive(Clone, Copy)]
pub struct Pos<T: Clone + Copy + Debug> {
    pub x: T,
    pub y: T,
}

impl<T: Clone + Copy + Debug> Pos<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Clone + Copy + Debug> Debug for Pos<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entry(&self.x).entry(&self.y).finish()
    }
}

/// Represents a tile in a (x,y) position inside a tilemap
#[derive(Clone)]
pub struct MapTile {
    pub pos: Pos<i32>,
    pub tileset: String,
    pub tileuid: u32,
    pub flip_x: bool,
}

impl Debug for MapTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.tileuid))
    }
}

/// Mainly used to customize the Debug, this is the list of tiles inside a layer.
///
/// The debug skips most of the data and ensures it is written in a single compact line.
#[derive(Clone)]
pub struct MapTileList {
    pub v: Vec<MapTile>,
}

impl Debug for MapTileList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.v))
    }
}

/// Possible layer types. We only care about Tiles and Group layers.
#[derive(Debug, Clone)]
pub enum MapLayerType {
    Tiles(MapTileList),
    Objects(),
    Image(),
    Group(MapLayerGroup),
}

/// A layer from a Tiled tilemap.
#[derive(Debug, Clone)]
pub struct MapLayer {
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
    pub offset: Pos<f32>,
    pub user_class: Option<String>,
    pub user_properties: HashMap<String, tiled::PropertyValue>,
    pub data: MapLayerType,
}

/// Iterator for layers. This iterator will search recursively for layers of
/// type `Tiles` and return them iteratively.
///
/// Mostly used to convert a recursive search into a linear one.
pub struct IterMapLayerGroup<'a> {
    pub iter: Vec<Iter<'a, MapLayer>>,
}

impl<'a> Iterator for IterMapLayerGroup<'a> {
    type Item = &'a MapLayer;

    fn next(&mut self) -> Option<Self::Item> {
        let op_iter = self.iter.pop();
        if let Some(mut iter) = op_iter {
            if let Some(layer) = iter.next() {
                self.iter.push(iter);
                match &layer.data {
                    MapLayerType::Tiles(_) => Some(layer),
                    MapLayerType::Objects() => self.next(),
                    MapLayerType::Image() => self.next(),
                    MapLayerType::Group(g) => {
                        self.iter.push(g.layers.iter());
                        self.next()
                    }
                }
            } else {
                self.next()
            }
        } else {
            None
        }
    }
}

/// Represents a Layer Group in Tiled. It is also used to provide the `iter()`
/// method to search recursively.
#[derive(Debug, Clone)]
pub struct MapLayerGroup {
    pub layers: Vec<MapLayer>,
}

impl MapLayerGroup {
    pub fn iter(&self) -> IterMapLayerGroup {
        IterMapLayerGroup {
            iter: vec![self.layers.iter()],
        }
    }
}

// ----------- Load functions -------------------

/// Entry point for loading tiled maps. 
/// 
/// Example:
///     let mut loader = tiled::Loader::new(); 
///     let map = loader.load_tmx_map("assets/maps/map_house1_3x.tmx").unwrap();
///     let map_layers = load_tile_layer_iter(map.layers());
pub fn load_tile_layer_iter<'a>(
    layer_iter: impl ExactSizeIterator<Item = tiled::Layer<'a>>,
) -> Vec<MapLayer> {
    let mut ret = vec![];
    for layer in layer_iter {
        let map_layer = MapLayer {
            name: layer.name.to_string(),
            visible: layer.visible,
            offset: Pos::new(layer.offset_x, layer.offset_y),
            user_class: layer.user_type.clone(),
            user_properties: layer.properties.clone(),
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
