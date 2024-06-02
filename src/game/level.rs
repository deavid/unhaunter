//! Level Management Module
//! -------------------------
//!
//! This module handles the loading and management of game levels, including:
//! * Loading TMX map data.
//! * Spawning Bevy entities for map tiles, players, ghosts, and other game objects.
//! * Initializing entity components based on TMX data and game logic.
//! * Managing room-related events and states, such as lighting conditions and interactive object behavior.
//! * Handling interactions between the player and interactive objects in the game world.
//!
//! This module provides the core functionality for setting up and managing the interactive environment
//! that the player explores and investigates.

use crate::behavior::component::RoomState;
use crate::behavior::Behavior;
use crate::board::{self, Bdl, BoardDataToRebuild, MapTileComponents, Position, SpriteDB};
use crate::components::ghost_influence::{GhostInfluence, InfluenceType};
use crate::game::{GameSound, MapUpdate, SoundType, SpriteType};
use crate::ghost::{GhostBreach, GhostSprite};
use crate::materials::CustomMaterial1;
use crate::player::{AnimationTimer, CharacterAnimation, InteractiveStuff, PlayerSprite};
use crate::root::{self, QuadCC};
use crate::tiledmap::{AtlasData, MapLayerType};
use crate::{behavior, gear, summary, tiledmap};
use bevy::prelude::*;
use bevy::sprite::{Anchor, MaterialMesh2dBundle};
use bevy::utils::hashbrown::HashMap;
use ordered_float::OrderedFloat;

use super::{GCameraArena, GameConfig, GameSprite};

/// Event triggered when the player enters a new room or when a significant room-related change occurs.
///
/// This event is used to trigger actions like opening the van UI or updating the state of interactive objects
/// based on the room's current state.
#[derive(Clone, Debug, Default, Event)]
pub struct RoomChangedEvent {
    /// Set to `true` if the event is triggered during level initialization.
    pub initialize: bool,
    /// Set to `true` if the van UI should be opened automatically (e.g., when the player returns to the starting area).
    pub open_van: bool,
}

impl RoomChangedEvent {
    /// Creates a new `RoomChangedEvent` specifically for level initialization.
    ///
    /// The `initialize` flag is set to `true`, and the `open_van` flag is set based on the given value.
    pub fn init(open_van: bool) -> Self {
        Self {
            initialize: true,
            open_van,
        }
    }
}

/// Event triggered to load a new level from a TMX map file.
///
/// This event initiates the level loading process, despawning existing entities, loading map data,
/// and spawning new entities based on the TMX file.
#[derive(Debug, Clone, Event)]
pub struct LoadLevelEvent {
    /// The file path to the TMX map file to be loaded.
    pub map_filepath: String,
}

/// Loads a new level based on the `LoadLevelEvent`.
///
/// This function despawns existing game entities, loads the specified TMX map, creates Bevy entities
/// for map tiles, players, ghosts, and other objects, initializes their components, and sets up game-related
/// resources, such as `BoardData`, `SpriteDB`, and `RoomDB`.
#[allow(clippy::too_many_arguments)]
pub fn load_level(
    mut ev: EventReader<LoadLevelEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut bf: ResMut<board::BoardData>,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
    qgs: Query<Entity, With<GameSprite>>,
    qgs2: Query<Entity, With<GameSound>>,
    mut ev_room: EventWriter<RoomChangedEvent>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tilesetdb: ResMut<tiledmap::MapTileSetDb>,
    mut sdb: ResMut<SpriteDB>,
    handles: Res<root::GameAssets>,
    mut roomdb: ResMut<board::RoomDB>,
    mut app_next_state: ResMut<NextState<root::State>>,
) {
    let mut ev_iter = ev.read();
    let Some(load_event) = ev_iter.next() else {
        return;
    };
    // Consume all events, just in case to prevent double loading.
    let _ = ev_iter.count();

    // Despawn sprites just in case
    for gs in qgs.iter() {
        commands.entity(gs).despawn_recursive();
    }
    // Despawn ambient sounds just in case.
    for gs in qgs2.iter() {
        commands.entity(gs).despawn_recursive();
    }
    // TODO: Ambient temp should probably come from either the map or be influenced by weather.
    bf.ambient_temp = 6.0;

    // Remove all pre-existing data for environment
    bf.temperature_field.clear();
    bf.sound_field.clear();
    roomdb.room_state.clear();
    roomdb.room_tiles.clear();

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
    commands
        .spawn(AudioBundle {
            source: asset_server.load("sounds/heartbeat-1.ogg"),
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
            class: SoundType::HeartBeat,
        });
    commands
        .spawn(AudioBundle {
            source: asset_server.load("sounds/insane-1.ogg"),
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
            class: SoundType::Insane,
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
    let mut van_entry_points: Vec<board::Position> = vec![];

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
    let mut movable_objects: Vec<Entity> = Vec::new();
    for (maptiles, layer) in layers.iter().filter_map(|(_, layer)| {
        // filter only the tile layers and extract that directly
        if let MapLayerType::Tiles(tiles) = &layer.data {
            Some((tiles, layer))
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
            let new_pos = Position {
                global_z: 0.0001,
                ..pos
            };
            match &mt.behavior.p.util {
                behavior::Util::PlayerSpawn => {
                    player_spawn_points.push(new_pos);
                }
                behavior::Util::GhostSpawn => {
                    ghost_spawn_points.push(new_pos);
                }
                behavior::Util::RoomDef(name) => {
                    roomdb
                        .room_tiles
                        .insert(pos.to_board_position(), name.to_owned());
                    roomdb.room_state.insert(name.clone(), behavior::State::Off);
                }
                behavior::Util::Van => {
                    van_entry_points.push(new_pos);
                }
                behavior::Util::None => {}
            }
            mt.behavior.default_components(&mut entity, layer);
            let mut beh = mt.behavior.clone();
            beh.flip(tile.flip_x);

            // --- Check if Object is Movable ---
            if mt.behavior.p.object.movable {
                // FIXME: It does not check if the item is in a valid room, since the rooms are still being constructed
                // .. at this point. This is something to fix later on.

                // --- Collect Movable Objects ---
                movable_objects.push(entity.id());
            }

            entity
                .insert(beh)
                .insert(GameSprite)
                .insert(pos)
                .insert(MapUpdate::default());
        }
    }

    use rand::seq::SliceRandom;
    use rand::thread_rng;
    let mut rng = thread_rng();

    // --- Map Validation ---
    if movable_objects.len() < 3 {
        warn!("Map '{}' has less than 3 movable objects in rooms. Ghost influence system might not work as intended.", load_event.map_filepath);
    }

    // --- Random Property Assignment ---

    if !movable_objects.is_empty() {
        // Shuffle the movable objects to ensure random selection
        movable_objects.shuffle(&mut rng);

        // Select up to 3 objects
        let selected_objects = movable_objects.iter().take(3);

        // Assign properties
        for (i, &entity) in selected_objects.enumerate() {
            let influence_type = if i == 0 {
                InfluenceType::Repulsive
            } else {
                InfluenceType::Attractive
            };

            // Add the GhostInfluence component to the selected entity
            commands.entity(entity).insert(GhostInfluence {
                influence_type,
                charge_value: 0.0,
            });
        }
    }

    player_spawn_points.shuffle(&mut thread_rng());
    if player_spawn_points.is_empty() {
        error!("No player spawn points found!! - that will probably not display the map because the player will be out of bounds");
    }
    let player_position = player_spawn_points.pop().unwrap();
    let player_scoord = player_position.to_screen_coord();

    let dist_to_van = van_entry_points
        .iter()
        .map(|v| OrderedFloat(v.distance(&player_position)))
        .min()
        .unwrap_or(OrderedFloat(1000.0))
        .into_inner();

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
    bf.breach_pos = ghost_spawn;
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

    let open_van: bool = dist_to_van < 4.0;
    ev_room.send(RoomChangedEvent::init(open_van));
}

/// Handles `RoomChangedEvent` events, updating interactive object states and room data.
///
/// This system is responsible for:
/// * Updating the state of interactive objects based on the current room's state.
/// * Triggering the opening of the van UI when appropriate (e.g., when the player enters the starting area).
/// * Updating the game's collision and lighting data after room-related changes.
pub fn roomchanged_event(
    mut ev_bdr: EventWriter<BoardDataToRebuild>,
    mut ev_room: EventReader<RoomChangedEvent>,
    mut interactive_stuff: InteractiveStuff,
    interactables: Query<(Entity, &board::Position, &Behavior, &RoomState), Without<PlayerSprite>>,
    gc: Res<GameConfig>,
    pc: Query<(&PlayerSprite, &Transform), Without<GCameraArena>>,
    mut camera: Query<&mut Transform, With<GCameraArena>>,
) {
    let Some(ev) = ev_room.read().next() else {
        return;
    };

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

    if ev.open_van {
        interactive_stuff
            .game_next_state
            .set(root::GameState::Truck);
    }
    if ev.initialize {
        for (player, p_transform) in pc.iter() {
            if player.id != gc.player_id {
                continue;
            }
            for mut cam_trans in camera.iter_mut() {
                cam_trans.translation = p_transform.translation;
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InteractionExecutionType {
    ChangeState,
    ReadRoomState,
}

pub fn app_setup(app: &mut App) {
    app.add_event::<RoomChangedEvent>()
        .add_event::<LoadLevelEvent>()
        .add_systems(Update, roomchanged_event)
        .add_systems(PostUpdate, load_level);
}
