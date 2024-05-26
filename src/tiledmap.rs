//! Functionality to load Tiled maps (tilesets and tilemaps) into Bevy for Unhaunter.
//!
//! Most of the classes here are almost a redefinition (for now) of the tiled library.
//! Currently serve as an example on how to load/store data.

use std::{fmt::Debug, slice::Iter};

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

// ------------ Bevy map loading utils --------------------

use crate::materials::CustomMaterial1;
use bevy::{prelude::*, utils::HashMap};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum AtlasData {
    Sheet((Handle<TextureAtlasLayout>, CustomMaterial1)),
    Tiles(Vec<(Handle<Image>, CustomMaterial1)>),
}

#[derive(Debug, Clone)]
pub struct MapTileSet {
    pub tileset: Arc<tiled::Tileset>,
    pub data: AtlasData,
    pub y_anchor: f32,
}

#[derive(Debug, Clone, Default, Resource)]
pub struct MapTileSetDb {
    pub db: HashMap<String, MapTileSet>,
}

#[cfg(not(target_arch = "wasm32"))]
mod arch {
    pub fn map_loader(path: impl AsRef<std::path::Path>) -> tiled::Map {
        let mut loader = tiled::Loader::new();
        loader.load_tmx_map(path).unwrap()
    }
}

#[cfg(target_arch = "wasm32")]
mod arch {
    use std::io::Cursor;

    /// Basic example reader impl that just keeps a few resources in memory
    struct MemoryReader;

    impl tiled::ResourceReader for MemoryReader {
        type Resource = Cursor<&'static [u8]>;
        type Error = std::io::Error;

        fn read_from(
            &mut self,
            path: &std::path::Path,
        ) -> std::result::Result<Self::Resource, Self::Error> {
            let path = path.to_str().unwrap();
            match path {
                "assets/maps/atut01_basics.tmx" => Ok(Cursor::new(include_bytes!(
                    "../assets/maps/atut01_basics.tmx"
                ))),
                "assets/maps/atut02_glass_house.tmx" => Ok(Cursor::new(include_bytes!(
                    "../assets/maps/atut02_glass_house.tmx"
                ))),
                "assets/maps/map_house1.tmx" => {
                    Ok(Cursor::new(include_bytes!("../assets/maps/map_house1.tmx")))
                }
                "assets/maps/map_house2.tmx" => {
                    Ok(Cursor::new(include_bytes!("../assets/maps/map_house2.tmx")))
                }
                "assets/maps/map_school1.tmx" => Ok(Cursor::new(include_bytes!(
                    "../assets/maps/map_school1.tmx"
                ))),
                "assets/maps/unhaunter_custom_tileset.tsx" => Ok(Cursor::new(include_bytes!(
                    "../assets/maps/unhaunter_custom_tileset.tsx"
                ))),
                "assets/maps/unhaunter_spritesheet2.tsx" => Ok(Cursor::new(include_bytes!(
                    "../assets/maps/unhaunter_spritesheet2.tsx"
                ))),
                "assets/maps/unhaunter_spritesheetA_3x3x3.tsx" => Ok(Cursor::new(include_bytes!(
                    "../assets/maps/unhaunter_spritesheetA_3x3x3.tsx"
                ))),
                "assets/maps/unhaunter_spritesheetA_6x6x10.tsx" => Ok(Cursor::new(include_bytes!(
                    "../assets/maps/unhaunter_spritesheetA_6x6x10.tsx"
                ))),
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "file not found",
                )),
            }
        }
    }
    pub fn map_loader(path: impl AsRef<std::path::Path>) -> tiled::Map {
        let mut loader =
            tiled::Loader::<tiled::DefaultResourceCache, MemoryReader>::with_cache_and_reader(
                tiled::DefaultResourceCache::new(),
                MemoryReader,
            );
        loader.load_tmx_map(path).unwrap()
    }
}

pub fn bevy_load_map(
    path: impl AsRef<std::path::Path>,
    asset_server: &AssetServer,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    tilesetdb: &mut ResMut<MapTileSetDb>,
) -> (tiled::Map, Vec<(usize, MapLayer)>) {
    // Parse Tiled file:

    let map = arch::map_loader(path);

    // Preload all tilesets referenced:
    for tileset in map.tilesets().iter() {
        // If an image is included, this is a tilemap. If no image is included this is a sprite collection.
        // Sprite collections are not supported right now.
        let data = if let Some(image) = &tileset.image {
            let img_src = image
                .source
                .to_str()
                .unwrap()
                .replace("assets/maps/../", "");
            // FIXME: When the images are loaded onto the GPU it seems that we need at least 1 pixel of empty space
            // .. so that the GPU can sample surrounding pixels properly.
            // .. This contrasts with how Tiled works, as it assumes a perfect packing if possible.
            const MARGIN: f32 = 0.8;
            // TODO: Ideally we would prefer to preload, upscale by nearest to 2x or 4x, and add a 2px margin. Recreating
            // .. the texture on the fly.
            let texture: Handle<Image> = asset_server.load(img_src);
            let rows = tileset.tilecount / tileset.columns;
            let atlas1 = TextureAtlasLayout::from_grid(
                Vec2::new(
                    tileset.tile_width as f32 + tileset.spacing as f32 - MARGIN,
                    tileset.tile_height as f32 + tileset.spacing as f32 - MARGIN,
                ),
                tileset.columns as usize,
                rows as usize,
                Some(Vec2::new(MARGIN, MARGIN)),
                Some(Vec2::new(MARGIN / 4.0, MARGIN / 2.0)),
            );
            let mut cmat = CustomMaterial1::from_texture(texture);
            cmat.data.sheet_rows = rows;
            cmat.data.sheet_cols = tileset.columns;
            cmat.data.sheet_idx = 0;
            cmat.data.sprite_width = tileset.tile_width as f32 + tileset.spacing as f32;
            cmat.data.sprite_height = tileset.tile_height as f32 + tileset.spacing as f32;

            let atlas1_handle = texture_atlases.add(atlas1);
            AtlasData::Sheet((atlas1_handle.clone(), cmat))
        } else {
            let mut images: Vec<(Handle<Image>, CustomMaterial1)> = vec![];
            for (_tileid, tile) in tileset.tiles() {
                // tile.collision
                if let Some(image) = &tile.image {
                    let img_src = image
                        .source
                        .to_str()
                        .unwrap()
                        .replace("assets/maps/../", "");
                    dbg!(&img_src);
                    let img_handle: Handle<Image> = asset_server.load(img_src);
                    let cmat = CustomMaterial1::from_texture(img_handle.clone());

                    images.push((img_handle, cmat));
                }
            }
            AtlasData::Tiles(images)
        };
        // NOTE: tile.offset_x/y is used when drawing, instead we want the center point.
        let anchor_bottom_px = tileset.properties.get("Anchor::bottom_px").and_then(|x| {
            if let tiled::PropertyValue::IntValue(n) = x {
                Some(n)
            } else {
                None
            }
        });

        let y_anchor: f32 = if let Some(n) = anchor_bottom_px {
            // find the fraction from the total image:
            let f = *n as f32 / (tileset.tile_height + tileset.spacing) as f32;
            // from the center:
            f - 0.5
        } else {
            -0.25
        };

        let mts = MapTileSet {
            tileset: tileset.clone(),
            data,
            y_anchor,
        };
        // Store the tileset in memory in case we need to do anything with it later on.
        if tilesetdb.db.insert(tileset.name.to_string(), mts).is_some() {
            eprintln!("ERROR: Already existing tileset loaded with name {:?} - make sure you don't have the same tileset loaded twice", tileset.name.to_string());
            // panic!();
        }
    }

    let map_layers = load_tile_layer_iter(map.layers());
    let grp = MapLayerGroup { layers: map_layers };

    let layers: Vec<(usize, MapLayer)> = grp
        .iter()
        .filter(|x| x.visible)
        .enumerate()
        .map(|(n, l)| (n, l.clone()))
        .collect();

    (map, layers)

    // let tile_size: (f32, f32) = (map.tile_width as f32, map.tile_height as f32);
    // bevy_load_layers(&layers, tile_size, &mut tilesetdb)
}

#[allow(dead_code)]
pub enum SpriteEnum {
    One(SpriteBundle),
    Sheet(SpriteSheetBundle),
}

#[allow(dead_code)]
pub fn bevy_load_layers(
    layers: &[(usize, MapLayer)],
    tile_size: (f32, f32),
    tilesetdb: &mut ResMut<MapTileSetDb>,
) -> Vec<(MapTile, SpriteEnum)> {
    let mut sprites = vec![];
    for (n, layer) in layers {
        if let MapLayerType::Tiles(tiles) = &layer.data {
            for tile in &tiles.v {
                // load tile
                let op_tileset = tilesetdb.db.get(&tile.tileset);
                if let Some(tileset) = op_tileset {
                    sprites.push((tile.clone(), bevy_load_tile(tile, tile_size, tileset, *n)));
                }
            }
        }
    }
    sprites
}

pub fn bevy_load_tile(
    tile: &MapTile,
    tile_size: (f32, f32),
    tileset: &MapTileSet,
    n: usize,
) -> SpriteEnum {
    use bevy::sprite::Anchor;
    let x = tile_size.0 * (tile.pos.x - tile.pos.y) as f32 / 2.0;
    let y = tile_size.1 * (-tile.pos.x - tile.pos.y) as f32 / 2.0;
    let z: f32 = n as f32 / 1000.0;
    let anchor = Anchor::Custom(Vec2::new(0.0, tileset.y_anchor));
    match &tileset.data {
        AtlasData::Sheet((handle, _opt_mat)) => SpriteEnum::Sheet(SpriteSheetBundle {
            atlas: TextureAtlas {
                layout: handle.clone(),
                index: tile.tileuid as usize,
            },
            sprite: Sprite {
                anchor,
                flip_x: tile.flip_x,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(x, y, z),
                ..default()
            },
            ..default()
        }),
        AtlasData::Tiles(v_img) => SpriteEnum::One(SpriteBundle {
            texture: v_img[tile.tileuid as usize].0.clone(),
            sprite: Sprite {
                flip_x: tile.flip_x,
                anchor,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(x, y, z),
                ..default()
            },
            ..default()
        }),
    }
}

/// Loads a TMX as text file and inspects the first lines to obtain class and display_name.
#[cfg(not(target_arch = "wasm32"))]
pub fn naive_tmx_loader(path: &str) -> anyhow::Result<(Option<String>, Option<String>)> {
    use std::io::BufRead as _;
    // <map version="1.10" tiledversion="1.10.2" class="UnhaunterMap1" orientation="isometric" renderorder="right-down" width="42" height="42" tilewidth="24" tileheight="12" infinite="0" nextlayerid="18" nextobjectid="15">
    //     <properties>
    //      <property name="display_name" value="123 Acorn Lane Street House"/>
    //     </properties>

    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
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
