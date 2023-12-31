use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use bevy::{ecs::system::adapter::dbg, sprite::Anchor};
use serde::de;
use tiled::{Loader, PropertyValue, TileLayer};

#[derive(Clone, Copy)]
struct Pos<T: Clone + Copy + Debug> {
    x: T,
    y: T,
}

impl<T: Clone + Copy + Debug> Pos<T> {
    fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Clone + Copy + Debug> Debug for Pos<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entry(&self.x).entry(&self.y).finish()
    }
}

#[derive(Clone)]
struct MapTile {
    pos: Pos<i32>,
    tileset: String,
    tileuid: u32,
}

impl Debug for MapTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.tileuid))
        // f.debug_struct("MapTile").field("x", &self.x).field("y", &self.y).field("tileset", &self.tileset).field("tileuid", &self.tileuid).finish()
    }
}

#[derive(Debug, Clone)]
struct MapLayer {
    name: String,
    offset: Pos<f32>,
    user_class: Option<String>,
    user_properties: HashMap<String, tiled::PropertyValue>,
    data: MapLayerType,
}
#[derive(Debug, Clone)]
struct MapLayerGroup {
    layers: Vec<MapLayer>,
}

#[derive(Debug, Clone)]
enum MapLayerType {
    Tiles(Vec<MapTile>),
    Objects(),
    Image(),
    Group(MapLayerGroup),
}

fn load_main() {
    let mut loader = Loader::new();
    let map = loader
        .load_tmx_map("assets/maps/map_house1_3x.tmx")
        .unwrap();
    // println!("{:?}", map);
    // println!("{:?}", map.tilesets()[0].get_tile(0).unwrap().probability);
    // let tileset = loader
    //     .load_tsx_tileset("assets/maps/unhaunter_spritesheet2.tsx")
    //     .unwrap();
    // assert_eq!(*map.tilesets()[0], tileset, "The tileset of the map should match with the expected tileset. If not this means that the map is loading the tileset from elsewhere");
    dbg!(map.width, map.height);
    dbg!(map.tile_width, map.tile_height);
    dbg!(map.infinite());
    assert!(!map.infinite(), "The tileset cannot be infinite!");
    dbg!(map.orientation);
    for (n, tileset) in map.tilesets().iter().enumerate() {
        dbg!(n, &tileset.name);
        dbg!(&tileset.image);
        // for (id, tile) in tileset.tiles() {
        //     dbg!(id);
        //     if let Some(collision) = &tile.collision {
        //         dbg!(collision.colour);
        //         for cobj in collision.object_data() {
        //             dbg!(cobj);
        //         }
        //     }
        // }
    }
    let map_layers = load_tile_layer_iter(map.layers());
    eprintln!("layers: {:?}", map_layers);
    println!("LOAD OK");
}

fn load_tile_layer_iter<'a>(
    layer_iter: impl ExactSizeIterator<Item = tiled::Layer<'a>>,
) -> Vec<MapLayer> {
    let mut ret = vec![];
    for layer in layer_iter {
        let map_layer = MapLayer {
            name: layer.name.to_string(),
            offset: Pos::new(layer.offset_x, layer.offset_y),
            user_class: layer.user_type.clone(),
            user_properties: layer.properties.clone(),
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

fn load_tile_layer_tiles(layer: tiled::TileLayer) -> Vec<MapTile> {
    let mut ret = vec![];

    for y in 0..layer.height().unwrap() as i32 {
        for x in 0..layer.width().unwrap() as i32 {
            let maybe_tile = layer.get_tile(x, y);

            if let Some(tile) = maybe_tile {
                let t = MapTile {
                    pos: Pos::new(x, y),
                    tileset: tile.get_tileset().name.to_string(),
                    tileuid: tile.id(),
                };
                ret.push(t);
            }
        }
    }
    eprintln!("{} tiles loaded", ret.len());
    ret
}

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(
            // This sets image filtering to nearest
            // This is done to prevent textures with low resolution (e.g. pixel art) from being blurred
            // by linear filtering.
            ImagePlugin::default_nearest(),
        ))
        .add_systems(Startup, setup)
        //        .add_systems(Update, sprite_movement)
        .add_systems(Update, camera_movement)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());

    // ------
    let mut loader = Loader::new();
    let map = loader
        .load_tmx_map("assets/maps/map_house1_3x.tmx")
        .unwrap();

    dbg!(map.width, map.height);
    dbg!(map.tile_width, map.tile_height);

    for (n, tileset) in map.tilesets().iter().enumerate() {
        dbg!(n, &tileset.name);
        dbg!(&tileset.image);
        // if &tileset.name != "A6x6x10" {
        //     continue;
        // }
        if let Some(image) = &tileset.image {
            let img_src = image
                .source
                .canonicalize()
                .expect("incorrect path on image source when loading TileSet")
                .to_string_lossy()
                .to_string();
            dbg!(&img_src);

            let rows = tileset.tilecount / tileset.columns;
            const MARGIN: f32 = 0.8;
            dbg!(&tileset.columns, rows);
            dbg!(&tileset.tile_width, &tileset.tile_height);
            dbg!(&tileset.offset_x, &tileset.offset_y);
            dbg!(&tileset.spacing);
            dbg!(&tileset.margin);
            let atlas1 = TextureAtlas::from_grid(
                asset_server.load(img_src),
                Vec2::new(
                    tileset.tile_width as f32 + tileset.spacing as f32 - MARGIN,
                    tileset.tile_height as f32 + tileset.spacing as f32 - MARGIN,
                ),
                tileset.columns as usize,
                rows as usize,
                Some(Vec2::new(MARGIN, MARGIN)),
                Some(Vec2::new(MARGIN / 4.0, MARGIN / 2.0)),
            );
            // NOTE: tile.offset_x/y is used when drawing, instead we want the center point.
            let anchor_bottom_px = tileset.properties.get("Anchor::bottom_px").and_then(|x| {
                if let PropertyValue::IntValue(n) = x {
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
            dbg!(y_anchor);
            let atlas1_handle = texture_atlases.add(atlas1);
            for (id, _tile) in tileset.tiles() {
                let mut id2 = TextureAtlasSprite::new(id as usize);
                let x = (id % tileset.columns) as f32 * tileset.tile_width as f32 * 2.1
                    - n as f32 * 900.0
                    - 100.0;
                let y = 300.0 - (id / tileset.columns) as f32 * tileset.tile_height as f32 * 2.1;
                // let old_anchor = id2.anchor.as_vec();
                // id2.anchor = Anchor::Custom(Vec2::new(old_anchor.x, old_anchor.y * 1.5));
                id2.anchor = Anchor::Custom(Vec2::new(0.0, y_anchor));
                commands.spawn(SpriteSheetBundle {
                    texture_atlas: atlas1_handle.clone(),
                    sprite: id2,
                    transform: Transform {
                        translation: Vec3::new(x, y, 0.0),
                        ..default()
                    },
                    ..default()
                });
                // Rectangle
                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgba(1.0, 0.1, 0.0, 0.1),
                        custom_size: Some(Vec2::new(4.0, 4.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(x, y, 0.01)),
                    ..default()
                });
                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgba(1.0, 0.1, 0.0, 0.2),
                        custom_size: Some(Vec2::new(2.0, 2.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(x, y, 0.02)),
                    ..default()
                });
                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgba(1.0, 0.1, 0.0, 0.4),
                        custom_size: Some(Vec2::new(1.0, 1.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(x, y, 0.03)),
                    ..default()
                });
            }
        }
    }
}

fn camera_movement(
    time: Res<Time>,
    mut camera_position: Query<(&mut Camera2d, &mut Transform)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    // const RADIUS: f32 = 300.0;
    // let phase = time.elapsed_seconds() / 10.0;

    let delta = time.delta_seconds() * 200.0;
    let mov = 2.0;
    let zoom = 1.0;
    for (_cam, mut transform) in camera_position.iter_mut() {
        if keyboard_input.pressed(KeyCode::D) {
            transform.translation.x += delta * transform.scale.x * mov;
        }
        if keyboard_input.pressed(KeyCode::A) {
            transform.translation.x -= delta * transform.scale.x * mov;
        }
        if keyboard_input.pressed(KeyCode::W) {
            transform.translation.y += delta * transform.scale.x * mov;
        }
        if keyboard_input.pressed(KeyCode::S) {
            transform.translation.y -= delta * transform.scale.x * mov;
        }
        if keyboard_input.pressed(KeyCode::Plus) {
            transform.scale /= f32::powf(1.003, delta * zoom);
        }
        if keyboard_input.pressed(KeyCode::Minus) {
            transform.scale *= f32::powf(1.003, delta * zoom);
        }
        if keyboard_input.pressed(KeyCode::Key1) {
            let z: f32 = 1.0;
            transform.scale = Vec3::new(z, z, z);
        }
        if keyboard_input.pressed(KeyCode::Key2) {
            let z: f32 = 1.0 / 2.0;
            transform.scale = Vec3::new(z, z, z);
        }
        if keyboard_input.pressed(KeyCode::Key3) {
            let z: f32 = 1.0 / 4.0;
            transform.scale = Vec3::new(z, z, z);
        }
        if keyboard_input.pressed(KeyCode::Key4) {
            let z: f32 = 1.0 / 8.0;
            transform.scale = Vec3::new(z, z, z);
        }
        // transform.translation.x = phase.cos() * RADIUS;
        // transform.translation.y = phase.sin() * RADIUS;
        // transform.scale = Vec3::new(0.5, 0.5, 0.5);
    }
}
