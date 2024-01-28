use crate::behavior::component::Interactive;
use crate::behavior::Behavior;
use crate::board::{Bdl, Direction, MapTileComponents, Position, SpriteDB};
use crate::materials::CustomMaterial1;
use crate::root::QuadCC;
use crate::tiledmap::{AtlasData, MapLayerType};
use crate::{behavior, tiledmap};
use crate::{
    board::{self, BoardDataToRebuild},
    root,
};
use bevy::ecs::system::SystemParam;
use bevy::sprite::{Anchor, MaterialMesh2dBundle};
use bevy::utils::hashbrown::HashMap;
use bevy::{prelude::*, render::camera::ScalingMode};
use std::time::Duration;

#[derive(Component)]
pub struct GCamera;

#[derive(Component, Debug)]
pub struct GameUI;

#[derive(Component, Debug)]
pub struct GameSprite;

#[derive(Component, Debug)]
pub struct GameSound {
    pub class: SoundType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SoundType {
    BackgroundHouse,
    BackgroundStreet,
}
#[derive(Component, Debug)]
pub struct PlayerSprite {
    pub id: usize,
    pub controls: ControlKeys,
    pub torch_enabled: bool,
}

/// Resource to know basic stuff of the game.
#[derive(Debug, Resource)]
pub struct GameConfig {
    /// Which player should the camera and lighting follow
    pub player_id: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self { player_id: 1 }
    }
}

impl PlayerSprite {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            controls: Self::default_controls(id),
            torch_enabled: true,
        }
    }
    pub fn default_controls(id: usize) -> ControlKeys {
        match id {
            1 => ControlKeys::WASD,
            2 => ControlKeys::IJKL,
            _ => ControlKeys::NONE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ControlKeys {
    pub up: KeyCode,
    pub down: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub activate: KeyCode,
    pub torch: KeyCode,
}

impl ControlKeys {
    const WASD: Self = ControlKeys {
        up: KeyCode::W,
        down: KeyCode::S,
        left: KeyCode::A,
        right: KeyCode::D,
        activate: KeyCode::E,
        torch: KeyCode::T,
    };
    const IJKL: Self = ControlKeys {
        up: KeyCode::I,
        down: KeyCode::K,
        left: KeyCode::J,
        right: KeyCode::L,
        activate: KeyCode::O,
        torch: KeyCode::T,
    };
    const NONE: Self = ControlKeys {
        up: KeyCode::Unlabeled,
        down: KeyCode::Unlabeled,
        left: KeyCode::Unlabeled,
        right: KeyCode::Unlabeled,
        activate: KeyCode::Unlabeled,
        torch: KeyCode::Unlabeled,
    };
}

pub fn setup(mut commands: Commands, qc: Query<Entity, With<GCamera>>) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }
    // 2D orthographic camera
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(200.0);
    commands.spawn(cam).insert(GCamera);
    info!("Game camera setup");
}

pub fn cleanup(
    mut commands: Commands,
    qc: Query<Entity, With<GCamera>>,
    qg: Query<Entity, With<GameUI>>,
    qgs: Query<Entity, With<GameSprite>>,
    qs: Query<Entity, With<GameSound>>,
) {
    // Despawn old camera if exists
    for cam in qc.iter() {
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

pub fn setup_ui(
    mut commands: Commands,
    handles: Res<root::GameAssets>,
    mut ev_load: EventWriter<LoadLevelEvent>,
) {
    // Spawn game UI
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                padding: UiRect {
                    left: Val::Percent(1.0),
                    right: Val::Percent(1.0),
                    top: Val::Percent(1.0),
                    bottom: Val::Percent(1.0),
                },
                ..default()
            },
            ..default()
        })
        .insert(GameUI)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
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
        });
    info!("Game UI loaded");
    // Spawn Player 1
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: handles.images.character1.clone(),
            sprite: TextureAtlasSprite {
                anchor: Anchor::Custom(handles.anchors.grid1x1x4),
                ..Default::default()
            },
            ..default()
        })
        .insert(GameSprite)
        .insert(PlayerSprite::new(1))
        .insert(Position::new_i64(-1, 0, 0).into_global_z(0.0005))
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
    ev_load.send(LoadLevelEvent {
        map_filepath: "default.json".to_string(),
    });
}

pub fn keyboard(
    app_state: Res<State<root::State>>,
    mut app_next_state: ResMut<NextState<root::State>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut camera: Query<&mut Transform, With<GCamera>>,
    gc: Res<GameConfig>,
    pc: Query<(&PlayerSprite, &Transform, &board::Direction), Without<GCamera>>,
) {
    if *app_state.get() != root::State::InGame {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_next_state.set(root::State::MainMenu);
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
            transform.translation += vector / RED;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.x += 2.0;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.x -= 2.0;
        }
        if keyboard_input.pressed(KeyCode::Up) {
            transform.translation.y += 2.0;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            transform.translation.y -= 2.0;
        }
        if keyboard_input.pressed(KeyCode::NumpadAdd) {
            transform.scale.x /= 1.02;
            transform.scale.y /= 1.02;
        }
        if keyboard_input.pressed(KeyCode::NumpadSubtract) {
            transform.scale.x *= 1.02;
            transform.scale.y *= 1.02;
        }
    }
}

#[derive(SystemParam)]
pub struct CollisionHandler<'w> {
    bf: Res<'w, board::BoardData>,
}

impl<'w> CollisionHandler<'w> {
    const ENABLE_COLLISION: bool = true;
    const PILLAR_SZ: f32 = 0.3;
    const PLAYER_SZ: f32 = 0.5;

    fn delta(&self, pos: &Position) -> Vec3 {
        let bpos = pos.to_board_position();
        let mut delta = Vec3::ZERO;
        for npos in bpos.xy_neighbors(1) {
            let cf = self
                .bf
                .collision_field
                .get(&npos)
                .copied()
                .unwrap_or_default();
            if !cf.free && Self::ENABLE_COLLISION {
                let dpos = npos.to_position().to_vec3() - pos.to_vec3();
                let mut dapos = dpos.abs();
                dapos.x -= Self::PILLAR_SZ;
                dapos.y -= Self::PILLAR_SZ;
                dapos.x = dapos.x.max(0.0);
                dapos.y = dapos.y.max(0.0);
                let ddist = dapos.distance(Vec3::ZERO);
                if ddist < Self::PLAYER_SZ {
                    if dpos.x < 0.0 {
                        dapos.x *= -1.0;
                    }
                    if dpos.y < 0.0 {
                        dapos.y *= -1.0;
                    }
                    let fix_dist = (Self::PLAYER_SZ - ddist).powi(2);
                    let dpos_fix = dapos / (ddist + 0.000001) * fix_dist;
                    delta += dpos_fix;
                }
            }
        }
        delta
    }
}

pub fn keyboard_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut players: Query<(
        &mut board::Position,
        &mut board::Direction,
        &mut PlayerSprite,
        &mut AnimationTimer,
    )>,
    mut commands: Commands,
    mut asset_server: Res<AssetServer>,
    colhand: CollisionHandler,
    interactables: Query<
        (Entity, &board::Position, &Interactive, &Behavior),
        Without<PlayerSprite>,
    >,
    mut interactive_stuff: InteractiveStuff,
) {
    const PLAYER_SPEED: f32 = 0.04;
    const DIR_MIN: f32 = 5.0;
    const DIR_MAX: f32 = 80.0;
    const DIR_STEPS: f32 = 15.0;
    const DIR_MAG2: f32 = DIR_MAX / DIR_STEPS;
    const DIR_RED: f32 = 1.001;
    for (mut pos, mut dir, mut player, mut anim) in players.iter_mut() {
        let col_delta = colhand.delta(&pos);
        pos.x -= col_delta.x;
        pos.y -= col_delta.y;

        let mut d = Direction {
            dx: 0.0,
            dy: 0.0,
            dz: 0.0,
        };

        if keyboard_input.pressed(player.controls.up) {
            d.dy += 1.0;
        }
        if keyboard_input.pressed(player.controls.down) {
            d.dy -= 1.0;
        }
        if keyboard_input.pressed(player.controls.left) {
            d.dx -= 1.0;
        }
        if keyboard_input.pressed(player.controls.right) {
            d.dx += 1.0;
        }
        if keyboard_input.just_pressed(player.controls.torch) {
            player.torch_enabled = !player.torch_enabled;
            let sound_file = match player.torch_enabled {
                true => "sounds/switch-on-1.ogg",
                false => "sounds/switch-off-1.ogg",
            };
            commands.spawn(AudioBundle {
                source: asset_server.load(sound_file),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Once,
                    volume: bevy::audio::Volume::Relative(bevy::audio::VolumeLevel::new(1.0)),
                    speed: 1.0,
                    paused: false,
                    spatial: false,
                },
            });
        }

        d = d.normalized();
        let col_delta_n = (col_delta * 100.0).clamp_length_max(1.0);
        let col_dotp = (d.dx * col_delta_n.x + d.dy * col_delta_n.y).clamp(0.0, 1.0);
        d.dx -= col_delta_n.x * col_dotp;
        d.dy -= col_delta_n.y * col_dotp;

        let delta = d / 0.1 + dir.normalized() / DIR_MAG2 / 1000.0;
        let dscreen = delta.to_screen_coord();
        anim.set_range(CharacterAnimation::from_dir(dscreen.x, dscreen.y * 2.0).to_vec());

        // d.dx /= 1.5; // Compensate for the projection

        pos.x += PLAYER_SPEED * d.dx;
        pos.y += PLAYER_SPEED * d.dy;
        dir.dx += DIR_MAG2 * d.dx;
        dir.dy += DIR_MAG2 * d.dy;

        let dir_dist = (dir.dx.powi(2) + dir.dy.powi(2)).sqrt();
        if dir_dist > DIR_MAX {
            dir.dx *= DIR_MAX / dir_dist;
            dir.dy *= DIR_MAX / dir_dist;
        } else if dir_dist > DIR_MIN {
            dir.dx /= DIR_RED;
            dir.dy /= DIR_RED;
        }

        // ----
        if keyboard_input.just_pressed(player.controls.activate) {
            // let d = dir.normalized();
            let mut max_dist = 1.4;
            let mut selected_entity = None;
            for (entity, item_pos, interactive, behavior) in interactables.iter() {
                let cp_delta = interactive.control_point_delta(behavior);
                let old_dist = pos.delta(*item_pos);
                let item_pos = Position {
                    x: item_pos.x + cp_delta.x,
                    y: item_pos.y + cp_delta.y,
                    z: item_pos.z + cp_delta.z,
                    global_z: item_pos.global_z,
                };
                let new_dist = pos.delta(item_pos);
                // let new_dist_norm = new_dist.normalized();
                // let dot_p = (new_dist_norm.dx * -d.dx + new_dist_norm.dy * -d.dy).clamp(0.0, 1.0);
                // let dref = new_dist + (&d * (new_dist.distance().min(1.0) * dot_p));
                let dref = new_dist;
                let dist = dref.distance();
                if dist < 1.5 {
                    dbg!(cp_delta, old_dist, new_dist, dref, dist);
                }
                if dist < max_dist {
                    max_dist = dist + 0.00001;
                    selected_entity = Some(entity);
                }
            }
            if let Some(entity) = selected_entity {
                for (entity, _, interactive, behavior) in
                    interactables.iter().filter(|(e, _, _, _)| *e == entity)
                {
                    interactive_stuff.execute_interaction(entity, interactive, behavior);
                }
            }
        }
    }
}

#[derive(SystemParam)]
pub struct InteractiveStuff<'w, 's> {
    bf: Res<'w, board::SpriteDB>,
    commands: Commands<'w, 's>,
    materials1: ResMut<'w, Assets<CustomMaterial1>>,
    asset_server: Res<'w, AssetServer>,
    ev_bdr: EventWriter<'w, BoardDataToRebuild>,
}

impl<'w, 's> InteractiveStuff<'w, 's> {
    fn execute_interaction(
        &mut self,
        entity: Entity,
        interactive: &Interactive,
        behavior: &Behavior,
    ) {
        let tuid = behavior.key_tuid();
        let cvo = behavior.key_cvo();
        let mut e_commands = self.commands.get_entity(entity).unwrap();
        for other_tuid in self.bf.cvo_idx.get(&cvo).unwrap().iter() {
            if *other_tuid == tuid {
                continue;
            }
            let other = self.bf.map_tile.get(other_tuid).unwrap();
            match other.bundle.clone() {
                Bdl::Mmb(b) => {
                    let mat = self.materials1.get(b.material).unwrap().clone();
                    let mat = self.materials1.add(mat);

                    e_commands.insert(mat);
                }
                Bdl::Sb(b) => {
                    e_commands.insert(b);
                }
            };
            let sound_file = interactive.sound_for_moving_into_state(&other.behavior);
            e_commands.insert(other.behavior.clone());
            self.commands.spawn(AudioBundle {
                source: self.asset_server.load(sound_file),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Once,
                    volume: bevy::audio::Volume::Relative(bevy::audio::VolumeLevel::new(1.0)),
                    speed: 1.0,
                    paused: false,
                    spatial: false,
                },
            });
            self.ev_bdr.send(BoardDataToRebuild {
                lighting: true,
                collision: true,
            });

            break;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CharacterAnimationDirection {
    NN,
    NW,
    WW,
    SW,
    SS,
    SE,
    EE,
    NE,
}

impl CharacterAnimationDirection {
    fn from_dir(dx: f32, dy: f32) -> Self {
        let dst = (dx * dx + dy * dy).sqrt() + 0.0000000001;
        let dx = (dx / dst).round() as i32;
        let dy = (dy / dst).round() as i32;
        match dx {
            1 => match dy {
                1 => Self::NE,
                -1 => Self::SE,
                _ => Self::EE,
            },
            0 => match dy {
                1 => Self::NN,
                -1 => Self::SS,
                _ => Self::SS,
            },
            -1 => match dy {
                1 => Self::NW,
                -1 => Self::SW,
                _ => Self::WW,
            },
            _ => Self::EE,
        }
    }
    fn to_delta_idx(self) -> usize {
        match self {
            Self::NN => 0,
            Self::NW => 1,
            Self::WW => 2,
            Self::SW => 3,
            Self::SS => 16,
            Self::SE => 17,
            Self::EE => 18,
            Self::NE => 19,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CharacterAnimationState {
    Standing,
    Walking,
}

impl CharacterAnimationState {
    fn to_delta_idx(self) -> usize {
        match self {
            Self::Standing => 32,
            Self::Walking => 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CharacterAnimation {
    state: CharacterAnimationState,
    dir: CharacterAnimationDirection,
}

impl CharacterAnimation {
    fn from_dir(dx: f32, dy: f32) -> Self {
        let dst = (dx * dx + dy * dy).sqrt() + 0.001;
        let state = if dst > 1.0 {
            CharacterAnimationState::Walking
        } else {
            CharacterAnimationState::Standing
        };
        let dir = CharacterAnimationDirection::from_dir(dx, dy);
        Self { state, dir }
    }
    fn to_vec(self) -> Vec<usize> {
        let i = self.state.to_delta_idx() + self.dir.to_delta_idx();
        vec![i, i + 4, i + 8, i + 12]
    }
}

#[derive(Component)]
pub struct AnimationTimer {
    timer: Timer,
    // range: RangeInclusive<usize>,
    frames: Vec<usize>,
    idx: usize,
}

impl AnimationTimer {
    pub fn from_range<I: IntoIterator<Item = usize>>(timer: Timer, range: I) -> Self {
        let frames: Vec<usize> = range.into_iter().collect();
        AnimationTimer {
            timer,
            frames,
            idx: 0,
        }
    }
    pub fn set_range<I: IntoIterator<Item = usize>>(&mut self, range: I) {
        self.frames = range.into_iter().collect();
    }
    pub fn tick(&mut self, delta: Duration) -> Option<usize> {
        self.timer.tick(delta);
        if !self.timer.just_finished() {
            return None;
        }
        self.idx = (self.idx + 1) % self.frames.len();
        Some(self.frames[self.idx])
    }
}

pub fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut anim, mut sprite, texture_atlas_handle) in query.iter_mut() {
        if let Some(idx) = anim.tick(time.delta()) {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = idx;
            if sprite.index >= texture_atlas.textures.len() {
                error!(
                    "sprite index {} out of range [0..{}]",
                    sprite.index,
                    texture_atlas.textures.len()
                );
            }
        }
    }
}

pub fn player_coloring(
    mut players: Query<(&mut TextureAtlasSprite, &PlayerSprite, &board::Position)>,
    bf: Res<board::BoardData>,
) {
    for (mut tas, player, position) in players.iter_mut() {
        let color: Color = match player.id {
            1 => Color::WHITE,
            2 => Color::GOLD,
            _ => Color::ORANGE_RED,
        };
        let bpos = position.to_board_position();
        // mapping of... distance vs rel_lux
        let mut tot_rel_lux = 0.0000001;
        let mut n_rel_lux = 0.0000001;
        for npos in bpos.xy_neighbors(2) {
            if let Some(lf) = bf.light_field.get(&npos) {
                let npos = npos.to_position();
                let dist = npos.distance(position);
                let f = (1.0 - dist).clamp(0.0, 1.0);
                let rel_lux = lf.lux / bf.current_exposure;
                n_rel_lux += f;
                tot_rel_lux += rel_lux * f;
            }
        }
        let rel_lux = tot_rel_lux / n_rel_lux;
        tas.color = board::compute_color_exposure(rel_lux, 0.0, board::DARK_GAMMA, color);
    }
}

#[derive(Debug, Clone, Event)]
pub struct LoadLevelEvent {
    map_filepath: String,
}

#[allow(clippy::too_many_arguments)]
pub fn load_level(
    mut ev: EventReader<LoadLevelEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
    qgs: Query<Entity, With<Behavior>>,
    mut qp: Query<&mut board::Position, With<PlayerSprite>>,
    mut ev_bdr: EventWriter<BoardDataToRebuild>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tilesetdb: ResMut<tiledmap::MapTileSetDb>,
    mut sdb: ResMut<SpriteDB>,
) {
    let Some(load_event) = ev.read().next() else {
        return;
    };

    for gs in qgs.iter() {
        commands.entity(gs).despawn_recursive();
    }
    commands
        .spawn(AudioBundle {
            source: asset_server.load("sounds/background-noise-house-1.ogg"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::Relative(bevy::audio::VolumeLevel::new(0.00001)),
                speed: 1.0,
                paused: false,
                spatial: false,
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
                volume: bevy::audio::Volume::Relative(bevy::audio::VolumeLevel::new(0.00001)),
                speed: 1.0,
                paused: false,
                spatial: false,
            },
        })
        .insert(GameSound {
            class: SoundType::BackgroundStreet,
        });
    dbg!(&load_event.map_filepath);
    commands.init_resource::<board::BoardData>();

    info!("Load Level");

    // ---------- NEW MAP LOAD ----------
    let (_map, layers) = tiledmap::bevy_load_map(
        "assets/maps/map_house1_3x.tmx",
        asset_server,
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
            match mt.behavior.p.util {
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
                _ => {}
            }
            mt.behavior.default_components(&mut entity);

            entity
                .insert(mt.behavior.clone())
                .insert(GameSprite)
                .insert(pos);
        }
    }

    use rand::seq::SliceRandom;
    use rand::thread_rng;
    player_spawn_points.shuffle(&mut thread_rng());
    if player_spawn_points.is_empty() {
        error!("No player spawn points found!! - that will probably not display the map because the player will be out of bounds");
    }
    for mut pos in qp.iter_mut() {
        if let Some(spawn) = player_spawn_points.pop() {
            *pos = spawn;
        }
    }

    ghost_spawn_points.shuffle(&mut thread_rng());
    if ghost_spawn_points.is_empty() {
        error!("No ghost spawn points found!! - that will probably break the gameplay as the ghost will spawn out of bounds");
    }
    // TODO: Spawn the ghost here / Set ghost initial position.

    ev_bdr.send(BoardDataToRebuild {
        lighting: true,
        collision: true,
    });
}
