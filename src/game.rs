use crate::behavior::component::RoomState;
use crate::behavior::Behavior;
use crate::board::{Bdl, MapTileComponents, Position, SpriteDB};
use crate::ghost::{GhostBreach, GhostSprite};
use crate::materials::CustomMaterial1;
use crate::player::{AnimationTimer, CharacterAnimation, InteractiveStuff, PlayerSprite};
use crate::root::QuadCC;
use crate::tiledmap::{AtlasData, MapLayerType};
use crate::{behavior, gear, summary, tiledmap};
use crate::{
    board::{self, BoardDataToRebuild},
    root,
};
use bevy::render::view::RenderLayers;
use bevy::sprite::{Anchor, MaterialMesh2dBundle};
use bevy::utils::hashbrown::HashMap;
use bevy::{prelude::*, render::camera::ScalingMode};

#[derive(Component)]
pub struct GCameraArena;

#[derive(Component)]
pub struct GCameraUI;

#[derive(Component, Debug)]
pub struct GameUI;

#[derive(Component, Debug)]
pub struct GameSprite;

#[derive(Component, Debug, Default)]
pub struct MapUpdate {
    pub last_update: f32,
}

#[derive(Component, Debug)]
pub struct GameSound {
    pub class: SoundType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SoundType {
    BackgroundHouse,
    BackgroundStreet,
}

#[derive(Clone, Debug, Default, Event)]
pub struct RoomChangedEvent;
/// Resource to know basic stuff of the game.
#[derive(Debug, Resource)]
pub struct GameConfig {
    /// Which player should the camera and lighting follow
    pub player_id: usize,
    pub right_hand_status_text: String,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            player_id: 1,
            right_hand_status_text: "".into(),
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub enum SpriteType {
    Ghost,
    Breach,
    Player,
    #[default]
    Other,
}

pub fn app_setup(app: &mut App) {
    app.init_resource::<GameConfig>()
        .add_event::<RoomChangedEvent>()
        .add_event::<LoadLevelEvent>()
        .add_systems(Update, roomchanged_event)
        .add_systems(OnEnter(root::State::InGame), setup)
        .add_systems(OnEnter(root::State::InGame), setup_ui)
        .add_systems(OnExit(root::State::InGame), cleanup)
        .add_systems(OnEnter(root::GameState::None), resume)
        .add_systems(OnExit(root::GameState::None), pause)
        .add_systems(Update, keyboard)
        .add_systems(PostUpdate, load_level);
}

pub fn setup(
    mut commands: Commands,
    qc: Query<Entity, With<GCameraArena>>,
    qc2: Query<Entity, With<GCameraUI>>,
) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }
    for cam in qc2.iter() {
        commands.entity(cam).despawn_recursive();
    }
    // 2D orthographic camera - Arena
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(200.0);
    commands
        .spawn(cam)
        .insert(GCameraArena)
        .insert(RenderLayers::from_layers(&[0, 1]));

    // 2D orthographic camera - UI
    let cam = Camera2dBundle {
        camera_2d: Camera2d,
        camera: Camera {
            // renders after / on top of the main camera
            order: 1,
            ..default()
        },
        ..default()
    };
    commands
        .spawn(cam)
        .insert(GCameraUI)
        .insert(RenderLayers::from_layers(&[2, 3]));
    info!("Game camera setup");
}

pub fn cleanup(
    mut commands: Commands,
    qc: Query<Entity, With<GCameraArena>>,
    qc2: Query<Entity, With<GCameraUI>>,
    qg: Query<Entity, With<GameUI>>,
    qgs: Query<Entity, With<GameSprite>>,
    qs: Query<Entity, With<GameSound>>,
) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }
    for cam in qc2.iter() {
        commands.entity(cam).despawn_recursive();
    }
    // Despawn game UI if not used
    for gui in qg.iter() {
        commands.entity(gui).despawn_recursive();
    }
    // Despawn game sprites if not used
    for gs in qgs.iter() {
        commands.entity(gs).despawn_recursive();
    }
    // Despawn game sound
    for gs in qs.iter() {
        commands.entity(gs).despawn_recursive();
    }
}

pub fn pause(mut qg: Query<&mut Visibility, With<GameUI>>) {
    for mut vis in qg.iter_mut() {
        *vis = Visibility::Hidden;
    }
}

pub fn resume(mut qg: Query<&mut Visibility, With<GameUI>>) {
    for mut vis in qg.iter_mut() {
        *vis = Visibility::Visible;
    }
}

pub fn setup_ui(mut commands: Commands, handles: Res<root::GameAssets>) {
    const DEBUG_BCOLOR: BorderColor = BorderColor(Color::rgba(0.0, 1.0, 1.0, 0.0003));
    const INVENTORY_STATS_COLOR: Color = Color::rgba(0.7, 0.7, 0.7, 0.6);
    const PANEL_BGCOLOR: Color = Color::rgba(0.1, 0.1, 0.1, 0.3);
    // Spawn game UI
    commands
        .spawn(NodeBundle {
            border_color: DEBUG_BCOLOR,

            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            ..default()
        })
        .insert(GameUI)
        .with_children(|parent| {
            // Top row (Game title)
            parent
                .spawn(NodeBundle {
                    border_color: DEBUG_BCOLOR,

                    style: Style {
                        border: UiRect::all(Val::Px(1.0)),
                        padding: UiRect::all(Val::Px(1.0)),
                        width: Val::Percent(20.0),
                        height: Val::Percent(5.0),
                        min_width: Val::Px(0.0),
                        min_height: Val::Px(16.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // logo
                    parent.spawn(ImageBundle {
                        style: Style {
                            aspect_ratio: Some(130.0 / 17.0),
                            width: Val::Percent(80.0),
                            height: Val::Auto,
                            max_width: Val::Percent(80.0),
                            max_height: Val::Percent(100.0),
                            flex_shrink: 1.0,
                            ..default()
                        },
                        image: handles.images.title.clone().into(),
                        ..default()
                    });
                });

            // Main game viewport - middle
            parent.spawn(NodeBundle {
                border_color: DEBUG_BCOLOR,
                style: Style {
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(1.0)),
                    flex_grow: 1.0,
                    min_height: Val::Px(2.0),
                    ..Default::default()
                },
                ..Default::default()
            });

            // Bottom side - inventory and stats
            parent
                .spawn(NodeBundle {
                    border_color: DEBUG_BCOLOR,
                    style: Style {
                        border: UiRect::all(Val::Px(1.0)),
                        padding: UiRect::all(Val::Px(1.0)),
                        height: Val::Px(100.0),
                        width: Val::Percent(99.9),
                        flex_direction: FlexDirection::Row,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    // Split for the bottom side in three regions

                    // Left side
                    parent
                        .spawn(NodeBundle {
                            border_color: DEBUG_BCOLOR,
                            style: Style {
                                border: UiRect::all(Val::Px(1.0)),
                                padding: UiRect::all(Val::Px(1.0)),
                                flex_grow: 1.0,
                                align_content: AlignContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            // For now a reminder of the keys:
                            let text_bundle = TextBundle::from_section(
                                "Movement: WASD - Interact: E\nToggle Aux: T - Toggle Main: R\nCycle Inv: Q - Swap: TAB",
                                TextStyle {
                                    font: handles.fonts.chakra.w300_light.clone(),
                                    font_size: 18.0,
                                    color: INVENTORY_STATS_COLOR,
                                },
                            );

                            parent.spawn(text_bundle);
                        });

                    // Mid side
                    parent.spawn(NodeBundle {
                        border_color: DEBUG_BCOLOR,
                        style: Style {
                            border: UiRect::all(Val::Px(1.0)),
                            padding: UiRect::all(Val::Px(1.0)),
                            flex_grow: 1.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    });

                    // Right side
                    parent
                        .spawn(NodeBundle {
                            border_color: DEBUG_BCOLOR,
                            background_color: BackgroundColor(PANEL_BGCOLOR),
                            style: Style {
                                border: UiRect::all(Val::Px(1.0)),
                                padding: UiRect::all(Val::Px(1.0)),
                                flex_grow: 1.0,
                                max_width: Val::Percent(33.3),
                                align_items: AlignItems::Center, // Vertical alignment
                                align_content: AlignContent::Start, // Horizontal alignment - start from the left.
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            // Right side panel - inventory
                            parent
                                .spawn(AtlasImageBundle {
                                    image: UiImage { texture: handles.images.gear.clone(), flip_x: false, flip_y: false },
                                    texture_atlas: TextureAtlas {
                                        index: gear::GearSpriteID::Flashlight2 as usize,
                                        layout: handles.images.gear_atlas.clone(),
                                    },
                                    ..default()
                                })
                                .insert(gear::playergear::Inventory::new_left());
                            parent
                                .spawn(AtlasImageBundle {
                                    image: UiImage { texture: handles.images.gear.clone(), flip_x: false, flip_y: false },
                                    texture_atlas: TextureAtlas {
                                        index: gear::GearSpriteID::IonMeter2 as usize,
                                        layout: handles.images.gear_atlas.clone(),
                                    },
                                    ..default()
                                })
                                .insert(gear::playergear::Inventory::new_right());
                            let mut text_bundle = TextBundle::from_section(
                                "-",
                                TextStyle {
                                    font: handles.fonts.victormono.w600_semibold.clone(),
                                    font_size: 20.0,
                                    color: INVENTORY_STATS_COLOR,
                                },
                            );
                            text_bundle.style = Style {
                                // width: Val::Px(200.0),
                                flex_grow: 1.0,
                                ..Default::default()
                            };
                            // text_bundle.background_color = BackgroundColor(PANEL_BGCOLOR);

                            parent.spawn(text_bundle).insert(gear::playergear::InventoryStats);
                        });
                });
        });

    info!("Game UI loaded");
}

#[allow(clippy::too_many_arguments)]
pub fn keyboard(
    app_state: Res<State<root::State>>,
    game_state: Res<State<root::GameState>>,
    // mut app_next_state: ResMut<NextState<root::State>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, With<GCameraArena>>,
    gc: Res<GameConfig>,
    pc: Query<(&PlayerSprite, &Transform, &board::Direction), Without<GCameraArena>>,
    time: Res<Time>,
) {
    if *app_state.get() != root::State::InGame {
        return;
    }
    if *game_state.get() != root::GameState::None {
        return;
    }
    let dt = time.delta_seconds() * 60.0;
    if keyboard_input.just_pressed(KeyCode::Escape) {
        game_next_state.set(root::GameState::Pause);
    }
    for mut transform in camera.iter_mut() {
        for (player, p_transform, p_dir) in pc.iter() {
            if player.id != gc.player_id {
                continue;
            }
            // Camera movement
            let mut ref_point = p_transform.translation;
            let sc_dir = p_dir.to_screen_coord();
            const CAMERA_AHEAD_FACTOR: f32 = 0.11;
            ref_point.y += 20.0 + sc_dir.y * CAMERA_AHEAD_FACTOR;
            ref_point.x += sc_dir.x * CAMERA_AHEAD_FACTOR;
            ref_point.z = transform.translation.z;
            let dist = (transform.translation.distance(ref_point) - 1.0).max(0.00001);
            let mut delta = ref_point - transform.translation;
            delta.z = 0.0;
            const RED: f32 = 120.0;
            const MEAN_DIST: f32 = 120.0;
            let vector = delta.normalize() * ((dist / MEAN_DIST).powf(2.2) * MEAN_DIST);
            transform.translation += vector / RED * dt;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            transform.translation.x += 2.0 * dt;
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= 2.0 * dt;
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            transform.translation.y += 2.0 * dt;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            transform.translation.y -= 2.0 * dt;
        }
        if keyboard_input.pressed(KeyCode::NumpadAdd) {
            transform.scale.x /= 1.02_f32.powf(dt);
            transform.scale.y /= 1.02_f32.powf(dt);
        }
        if keyboard_input.pressed(KeyCode::NumpadSubtract) {
            transform.scale.x *= 1.02_f32.powf(dt);
            transform.scale.y *= 1.02_f32.powf(dt);
        }
    }
}

#[derive(Debug, Clone, Event)]
pub struct LoadLevelEvent {
    pub map_filepath: String,
}

#[allow(clippy::too_many_arguments)]
pub fn load_level(
    mut ev: EventReader<LoadLevelEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut bf: ResMut<board::BoardData>,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
    qgs: Query<Entity, With<GameSprite>>,
    mut ev_room: EventWriter<RoomChangedEvent>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tilesetdb: ResMut<tiledmap::MapTileSetDb>,
    mut sdb: ResMut<SpriteDB>,
    handles: Res<root::GameAssets>,
    mut roomdb: ResMut<board::RoomDB>,
    mut camera: Query<&mut Transform, With<GCameraArena>>,
    mut app_next_state: ResMut<NextState<root::State>>,
) {
    let Some(load_event) = ev.read().next() else {
        return;
    };

    for gs in qgs.iter() {
        commands.entity(gs).despawn_recursive();
    }
    // TODO: Ambient temp should probably come from either the map or be influenced by weather.
    bf.ambient_temp = 6.0;

    // Remove all pre-existing data for environment
    bf.temperature_field.clear();
    bf.sound_field.clear();

    commands
        .spawn(AudioBundle {
            source: asset_server.load("sounds/background-noise-house-1.ogg"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::new(0.00001),
                speed: 1.0,
                paused: false,
                spatial: false,
                spatial_scale: None,
            },
        })
        .insert(GameSound {
            class: SoundType::BackgroundHouse,
        });
    commands
        .spawn(AudioBundle {
            source: asset_server.load("sounds/ambient-clean.ogg"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::new(0.00001),
                speed: 1.0,
                paused: false,
                spatial: false,
                spatial_scale: None,
            },
        })
        .insert(GameSound {
            class: SoundType::BackgroundStreet,
        });
    commands.init_resource::<board::BoardData>();

    info!("Load Level: {}", &load_event.map_filepath);
    app_next_state.set(root::State::InGame);

    // ---------- NEW MAP LOAD ----------
    let (_map, layers) = tiledmap::bevy_load_map(
        &load_event.map_filepath,
        &asset_server,
        &mut texture_atlases,
        &mut tilesetdb,
    );
    let mut player_spawn_points: Vec<board::Position> = vec![];
    let mut ghost_spawn_points: Vec<board::Position> = vec![];

    let mut mesh_tileset = HashMap::<String, Handle<Mesh>>::new();
    sdb.clear();

    // Load the tileset sprites first:
    for (tset_name, tileset) in tilesetdb.db.iter() {
        for (tileuid, tiled_tile) in tileset.tileset.tiles() {
            let anchor = Anchor::Custom(Vec2::new(0.0, tileset.y_anchor));
            let sprite_config =
                behavior::SpriteConfig::from_tiled_auto(tset_name.clone(), tileuid, &tiled_tile);
            let behavior = behavior::Behavior::from_config(sprite_config);
            let visibility = if behavior.p.display.disable {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
            let bundle = match &tileset.data {
                AtlasData::Sheet((handle, cmat)) => {
                    let mut cmat = cmat.clone();
                    let tatlas = texture_atlases.get(handle).unwrap();
                    let mesh_handle = mesh_tileset
                        .entry(tset_name.to_string())
                        .or_insert_with(|| {
                            let sprite_size = Vec2::new(
                                tatlas.size.x / cmat.data.sheet_cols as f32 * 1.005,
                                tatlas.size.y / cmat.data.sheet_rows as f32 * 1.005,
                            );
                            let sprite_anchor = Vec2::new(
                                sprite_size.x / 2.0,
                                sprite_size.y * (0.5 - tileset.y_anchor),
                            );
                            let base_quad = Mesh::from(QuadCC::new(sprite_size, sprite_anchor));
                            meshes.add(base_quad)
                        })
                        .clone();

                    cmat.data.sheet_idx = tileuid;
                    // Set alpha initially transparent to all materials so they will appear slowly.
                    cmat.data.color.set_a(0.0);
                    cmat.data.gamma = 0.1;
                    cmat.data.gbl = 0.1;
                    cmat.data.gbr = 0.1;
                    cmat.data.gtl = 0.1;
                    cmat.data.gtr = 0.1;
                    let mat = materials1.add(cmat);
                    let transform = Transform::from_xyz(-10000.0, -10000.0, -1000.0);
                    Bdl::Mmb(MaterialMesh2dBundle {
                        mesh: mesh_handle.into(),
                        material: mat.clone(),
                        transform,
                        visibility,
                        ..Default::default()
                    })
                }
                AtlasData::Tiles(v_img) => Bdl::Sb(SpriteBundle {
                    texture: v_img[tileuid as usize].0.clone(),
                    sprite: Sprite {
                        anchor,
                        ..default()
                    },
                    visibility,
                    transform: Transform::from_xyz(-10000.0, -10000.0, -1000.0),
                    ..default()
                }),
            };

            let key_tuid = behavior.key_tuid();
            sdb.cvo_idx
                .entry(behavior.key_cvo())
                .or_default()
                .push(key_tuid.clone());

            let mt = MapTileComponents { bundle, behavior };
            sdb.map_tile.insert(key_tuid, mt);
        }
    }
    // ----

    // We will need a 2nd pass load to sync some data
    // ----
    let mut c: f32 = 0.0;
    for maptiles in layers.iter().filter_map(|(_, layer)| {
        // filter only the tile layers and extract that directly
        if let MapLayerType::Tiles(tiles) = &layer.data {
            Some(tiles)
        } else {
            None
        }
    }) {
        for tile in &maptiles.v {
            let mt = sdb
                .map_tile
                .get(&(tile.tileset.clone(), tile.tileuid))
                .expect("Map references non-existent tileset+tileuid");
            // Spawn the base entity
            let mut entity = match &mt.bundle {
                Bdl::Mmb(b) => {
                    let mut b = b.clone();
                    if tile.flip_x {
                        b.transform.scale.x = -1.0;
                    }
                    let mat = materials1.get(b.material).unwrap().clone();
                    let mat = materials1.add(mat);

                    b.material = mat;
                    commands.spawn(b)
                }
                Bdl::Sb(b) => {
                    let mut b = b.clone();
                    if tile.flip_x {
                        b.transform.scale.x = -1.0;
                    }
                    commands.spawn(b.clone())
                }
            };

            let mut pos = board::Position {
                x: tile.pos.x as f32,
                y: -tile.pos.y as f32,
                z: 0.0,
                global_z: 0.0,
            };

            c += 0.000000001;
            pos.global_z = f32::from(mt.behavior.p.display.global_z) + c;
            match &mt.behavior.p.util {
                behavior::Util::PlayerSpawn => {
                    player_spawn_points.push(Position {
                        global_z: 0.0001,
                        ..pos
                    });
                }
                behavior::Util::GhostSpawn => {
                    ghost_spawn_points.push(Position {
                        global_z: 0.0001,
                        ..pos
                    });
                }
                behavior::Util::RoomDef(name) => {
                    roomdb
                        .room_tiles
                        .insert(pos.to_board_position(), name.to_owned());
                    roomdb.room_state.insert(name.clone(), behavior::State::Off);
                }
                behavior::Util::Van => {}
                behavior::Util::None => {}
            }
            mt.behavior.default_components(&mut entity);
            let mut beh = mt.behavior.clone();
            beh.flip(tile.flip_x);

            entity
                .insert(beh)
                .insert(GameSprite)
                .insert(pos)
                .insert(MapUpdate::default());
        }
    }

    use rand::seq::SliceRandom;
    use rand::thread_rng;
    player_spawn_points.shuffle(&mut thread_rng());
    if player_spawn_points.is_empty() {
        error!("No player spawn points found!! - that will probably not display the map because the player will be out of bounds");
    }
    let player_position = player_spawn_points.pop().unwrap();
    let player_scoord = player_position.to_screen_coord();

    for mut cam_trans in camera.iter_mut() {
        cam_trans.translation = player_scoord;
    }
    // Spawn Player 1
    commands
        .spawn(SpriteSheetBundle {
            texture: handles.images.character1.clone(),
            sprite: Sprite {
                anchor: Anchor::Custom(handles.anchors.grid1x1x4),
                ..default()
            },
            atlas: TextureAtlas {
                layout: handles.images.character1_atlas.clone(),
                ..Default::default()
            },
            transform: Transform::from_xyz(player_scoord[0], player_scoord[1], player_scoord[2])
                .with_scale(Vec3::new(0.5, 0.5, 0.5)),
            ..default()
        })
        .insert(GameSprite)
        .insert(gear::playergear::PlayerGear::new())
        .insert(PlayerSprite::new(1))
        .insert(SpriteType::Player)
        .insert(player_position)
        .insert(board::Direction::default())
        .insert(AnimationTimer::from_range(
            Timer::from_seconds(0.20, TimerMode::Repeating),
            CharacterAnimation::from_dir(0.5, 0.5).to_vec(),
        ));

    // Spawn Player 2
    // commands
    //     .spawn(SpriteSheetBundle {
    //         texture_atlas: handles.images.character1.clone(),
    //         sprite: TextureAtlasSprite {
    //             anchor: TileSprite::Character.anchor(&tb),
    //             ..Default::default()
    //         },
    //         ..default()
    //     })
    //     .insert(GameSprite)
    //     .insert(PlayerSprite::new(2))
    //     .insert(board::Direction::default())
    //     .insert(Position::new_i64(1, 0, 0).into_global_z(0.0005))
    //     .insert(AnimationTimer::from_range(
    //         Timer::from_seconds(0.20, TimerMode::Repeating),
    //         OldCharacterAnimation::Walking.animation_range(),
    //     ));
    bf.evidences.clear();

    ghost_spawn_points.shuffle(&mut thread_rng());

    if ghost_spawn_points.is_empty() {
        error!("No ghost spawn points found!! - that will probably break the gameplay as the ghost will spawn out of bounds");
    }
    let ghost_spawn = ghost_spawn_points.pop().unwrap();
    let ghost_sprite = GhostSprite::new(ghost_spawn.to_board_position());
    let ghost_types = vec![ghost_sprite.class];
    for evidence in ghost_sprite.class.evidences() {
        bf.evidences.insert(evidence);
    }
    commands.insert_resource(summary::SummaryData::new(ghost_types));
    let breach_id = commands
        .spawn(SpriteBundle {
            texture: asset_server.load("img/breach.png"),
            transform: Transform::from_xyz(-1000.0, -1000.0, -1000.0),
            sprite: Sprite {
                anchor: Anchor::Custom(handles.anchors.grid1x1x4),
                color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(GameSprite)
        .insert(SpriteType::Breach)
        .insert(GhostBreach)
        .insert(ghost_spawn)
        .id();

    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("img/ghost.png"),
            transform: Transform::from_xyz(-1000.0, -1000.0, -1000.0),
            sprite: Sprite {
                anchor: Anchor::Custom(handles.anchors.grid1x1x4),
                color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(GameSprite)
        .insert(SpriteType::Ghost)
        .insert(ghost_sprite.with_breachid(breach_id))
        .insert(ghost_spawn);

    ev_room.send(RoomChangedEvent);
}

pub fn roomchanged_event(
    mut ev_bdr: EventWriter<BoardDataToRebuild>,
    mut ev_room: EventReader<RoomChangedEvent>,
    mut interactive_stuff: InteractiveStuff,
    interactables: Query<(Entity, &board::Position, &Behavior, &RoomState), Without<PlayerSprite>>,
) {
    if ev_room.read().next().is_none() {
        return;
    }

    for (entity, item_pos, behavior, room_state) in interactables.iter() {
        let changed = interactive_stuff.execute_interaction(
            entity,
            item_pos,
            None,
            behavior,
            Some(room_state),
            InteractionExecutionType::ReadRoomState,
        );

        if changed {
            // dbg!(&behavior);
        }
    }
    ev_bdr.send(BoardDataToRebuild {
        lighting: true,
        collision: true,
    });
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InteractionExecutionType {
    ChangeState,
    ReadRoomState,
}
