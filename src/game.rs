use crate::board::{Direction, Position, TileSprite};
use crate::materials::CustomMaterial1;
use crate::tiledmap::SpriteEnum;
use crate::{
    board::{self, BoardDataToRebuild},
    root,
};
use crate::{levelparse, tiledmap};
use bevy::{prelude::*, render::camera::ScalingMode};
use std::time::Duration;

#[derive(Component)]
pub struct GCamera;

#[derive(Component, Debug)]
pub struct GameUI;

#[derive(Component, Debug)]
pub struct GameSprite;

#[derive(Component, Debug)]
pub struct PlayerSprite {
    pub id: usize,
    pub controls: ControlKeys,
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
}

impl ControlKeys {
    const WASD: Self = ControlKeys {
        up: KeyCode::W,
        down: KeyCode::S,
        left: KeyCode::A,
        right: KeyCode::D,
        activate: KeyCode::E,
    };
    const IJKL: Self = ControlKeys {
        up: KeyCode::I,
        down: KeyCode::K,
        left: KeyCode::J,
        right: KeyCode::L,
        activate: KeyCode::O,
    };
    const NONE: Self = ControlKeys {
        up: KeyCode::Unlabeled,
        down: KeyCode::Unlabeled,
        left: KeyCode::Unlabeled,
        right: KeyCode::Unlabeled,
        activate: KeyCode::Unlabeled,
    };
}

pub fn setup(mut commands: Commands, qc: Query<Entity, With<GCamera>>) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }
    // 2D orthographic camera
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(400.0);
    commands.spawn(cam).insert(GCamera);
    info!("Game camera setup");
}

pub fn cleanup(
    mut commands: Commands,
    qc: Query<Entity, With<GCamera>>,
    qg: Query<Entity, With<GameUI>>,
    qgs: Query<Entity, With<GameSprite>>,
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
}

pub fn setup_ui(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    handles: Res<root::GameAssets>,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
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
    let tb = board::TileBuilder::new(&images, &handles, &mut materials1);
    // Spawn Player 1
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: handles.images.character1.clone(),
            sprite: TextureAtlasSprite {
                anchor: TileSprite::Character.anchor(&tb),
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
            let mut ref_point = p_transform.translation;
            ref_point.y += 20.0 + p_dir.dy;
            ref_point.x += p_dir.dx;
            ref_point.z = transform.translation.z;
            let dist = transform.translation.distance(ref_point);
            let mut delta = ref_point - transform.translation;
            delta.z = 0.0;
            const RED: f32 = 20.0;
            const F: f32 = 10.0;
            let vector = delta / (0.1 + dist / F).sqrt();
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

pub fn keyboard_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut players: Query<(
        &mut board::Position,
        &mut Transform,
        &mut board::Direction,
        &PlayerSprite,
        &mut AnimationTimer,
    )>,
    bf: Res<board::BoardData>,
) {
    const PLAYER_SPEED: f32 = 0.04;
    const DIR_MIN: f32 = 5.0;
    const DIR_MAX: f32 = 80.0;
    const DIR_STEPS: f32 = 5.0;
    const DIR_MAG2: f32 = DIR_MAX / DIR_STEPS;
    const DIR_RED: f32 = 1.001;
    for (mut pos, mut transform, mut dir, player, mut anim) in players.iter_mut() {
        let bpos = pos.to_board_position();
        for npos in bpos.xy_neighbors(1) {
            let cf = bf.collision_field.get(&npos).copied().unwrap_or_default();
            if !cf.free {
                let dpos = npos.to_position().to_vec3() - pos.to_vec3();
                const PILLAR_SZ: f32 = 0.3;
                const PLAYER_SZ: f32 = 0.7;
                let mut dapos = dpos.abs();
                dapos.x -= PILLAR_SZ;
                dapos.y -= PILLAR_SZ;
                dapos.x = dapos.x.max(0.0);
                dapos.y = dapos.y.max(0.0);
                let ddist = dapos.distance(Vec3::ZERO);
                if ddist < PLAYER_SZ {
                    if dpos.x < 0.0 {
                        dapos.x *= -1.0;
                    }
                    if dpos.y < 0.0 {
                        dapos.y *= -1.0;
                    }
                    let fix_dist = (PLAYER_SZ - ddist).powi(2);
                    let dpos_fix = dapos / (ddist + 0.000001) * fix_dist;
                    pos.x -= dpos_fix.x;
                    pos.y -= dpos_fix.y;
                }
            }
        }
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
        d = d.normalized();

        let delta = d / 0.1 + dir.normalized() / DIR_MAG2 / 1000.0;
        let dscreen = delta.to_screen_coord();
        anim.set_range(CharacterAnimation::from_dir(dscreen.x, dscreen.y * 2.0).to_vec());
        transform.rotation = Quat::default();

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
    images: Res<Assets<Image>>,
    handles: Res<root::GameAssets>,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
    qgs: Query<Entity, With<board::Tile>>,
    mut qp: Query<&mut board::Position, With<PlayerSprite>>,
    mut ev_bdr: EventWriter<BoardDataToRebuild>,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut tilesetdb: ResMut<tiledmap::MapTileSetDb>,
) {
    let Some(load_event) = ev.read().next() else {
        return;
    };

    dbg!(&load_event.map_filepath);
    commands.init_resource::<board::BoardData>();

    info!("Load Level");

    // ---------- NEW MAP LOAD ----------
    let (map, layers) = tiledmap::bevy_load_map(
        "assets/maps/map_house1_3x.tmx",
        asset_server,
        texture_atlases,
        &mut tilesetdb,
    );
    let tile_size: (f32, f32) = (map.tile_width as f32, map.tile_height as f32);
    let sprites = tiledmap::bevy_load_layers(&layers, tile_size, &mut tilesetdb);

    for (tile, bundle) in sprites {
        /*
        -  tile: Tile,
        -        mut pos: Position,
        -        bundle: impl Bundle, -> GameSprite
        -        for_editor: bool,     -> false

               let sprite = match for_editor {
                   true => tile.sprite,
                   false => tile.sprite.as_displayed(),
               };
               pos.global_z = sprite.global_z();
               let bdl = self.custom_tile(sprite); // SpriteSheetBundle?
               let mut new_tile = commands.spawn(bdl);
               new_tile
                   .insert(bundle)
                   .insert(pos)
                   .insert(TileColor {
                       color: sprite.color(),
                   })
                   .insert(tile);
               */
        let pos = board::Position {
            x: tile.pos.x as f32 / 3.0,
            y: -tile.pos.y as f32 / 3.0,
            z: 0.0,
            global_z: 0.0,
        };

        let mut new_tile = match bundle {
            SpriteEnum::One(b) => commands.spawn(b),
            SpriteEnum::Sheet(b) => commands.spawn(b),
        };
        new_tile.insert(GameSprite).insert(pos);
    }

    // --------- OLD LEVEL LOAD ------------
    let json_u8 = std::fs::read("default_map.json").unwrap();
    let json = std::str::from_utf8(&json_u8).unwrap();
    let level = levelparse::Level::deserialize_json(json).unwrap();
    // Despawn tiles before loading the level
    for gs in qgs.iter() {
        commands.entity(gs).despawn_recursive();
    }
    let tb = board::TileBuilder::new(&images, &handles, &mut materials1);
    let mut spawn_points: Vec<board::Position> = vec![];
    for tile in level.tiles.iter() {
        let pos: board::Position = tile.position.into();
        if tile.sprite == board::TileSprite::Character {
            spawn_points.push(pos);
        }
        let tile = board::Tile {
            sprite: tile.sprite,
            variant: tile.variant,
        };
        // TODO: The IDs spawned are lost and can't be tracked in x,y coordinates
        // We need to store the entity ids into a hashmap.
        let _id = tb.spawn_tile(&mut commands, tile, pos, GameSprite, false);
    }
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    spawn_points.shuffle(&mut thread_rng());

    for mut pos in qp.iter_mut() {
        if let Some(spawn) = spawn_points.pop() {
            *pos = spawn;
        }
    }
    ev_bdr.send(BoardDataToRebuild {
        lighting: true,
        collision: true,
    });
}
