mod tiledmap;

use bevy::sprite::Anchor;
use std::{collections::HashMap, sync::Arc};
use tiled::{Loader, PropertyValue, Tileset};

use tiledmap::*;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(
            // This sets image filtering to nearest
            // This is done to prevent textures with low resolution (e.g. pixel art) from being blurred
            // by linear filtering.
            ImagePlugin::default_nearest(),
        ))
        .init_resource::<MapTileSetDb>()
        .add_systems(Startup, setup)
        //        .add_systems(Update, sprite_movement)
        .add_systems(Update, camera_movement)
        .run();
}

#[derive(Debug, Clone)]
struct MapTileSet {
    tileset: Arc<Tileset>,
    handle: Handle<TextureAtlas>,
    y_anchor: f32,
}

#[derive(Debug, Clone, Default, Resource)]
struct MapTileSetDb {
    db: HashMap<String, MapTileSet>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut tilesetdb: ResMut<MapTileSetDb>,
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
            let mts = MapTileSet {
                tileset: tileset.clone(),
                handle: atlas1_handle.clone(),
                y_anchor,
            };
            if tilesetdb.db.insert(tileset.name.to_string(), mts).is_some() {
                eprintln!("ERROR: Already existing tileset loaded with name {:?} - make sure you don't have the same tileset loaded twice", tileset.name.to_string());
                panic!();
            }
            const DEBUG_SPRITE_ANCHOR: bool = false;
            if DEBUG_SPRITE_ANCHOR {
                for (id, _tile) in tileset.tiles() {
                    let mut id2 = TextureAtlasSprite::new(id as usize);
                    let x = (id % tileset.columns) as f32 * tileset.tile_width as f32 * 2.1
                        - n as f32 * 900.0
                        - 100.0;
                    let y =
                        300.0 - (id / tileset.columns) as f32 * tileset.tile_height as f32 * 2.1;
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
    // end of tileset load

    let map_layers = load_tile_layer_iter(map.layers());
    let grp = MapLayerGroup { layers: map_layers };
    for (n, layer) in grp
        .iter()
        .filter(|x| x.visible && x.opacity > 0.9)
        .enumerate()
    {
        let z: f32 = n as f32 / 1000.0;
        if let MapLayerType::Tiles(tiles) = &layer.data {
            for tile in &tiles.v {
                let x = map.tile_width as f32 * (tile.pos.x - tile.pos.y) as f32 / 2.0;
                let y = map.tile_height as f32 * (-tile.pos.x - tile.pos.y) as f32 / 2.0;
                let op_tileset = tilesetdb.db.get(&tile.tileset);
                if let Some(tileset) = op_tileset {
                    let mut id = TextureAtlasSprite::new(tile.tileuid as usize);
                    id.anchor = Anchor::Custom(Vec2::new(0.0, tileset.y_anchor));
                    id.flip_x = tile.flip_x;
                    commands.spawn(SpriteSheetBundle {
                        texture_atlas: tileset.handle.clone(),
                        sprite: id,
                        transform: Transform {
                            translation: Vec3::new(x, y, z),
                            ..default()
                        },
                        ..default()
                    });
                }
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
