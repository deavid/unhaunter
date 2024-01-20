use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crate::{
    board::{self, BoardDataToRebuild, BoardPosition, CollisionFieldData, TileSprite, TileVariant},
    game, levelparse,
    materials::CustomMaterial1,
    root,
};
use bevy::{prelude::*, render::camera::ScalingMode, utils::HashMap};
use rand::Rng as _;

#[derive(Component, Debug)]
pub struct EditorUI;

#[derive(Component, Debug)]
pub struct EditorSprite;

#[derive(Component)]
pub struct ECamera;

#[derive(Component, Debug)]
pub struct SelectedPiece {
    pub piece: board::TileSprite,
}

#[derive(Component, Debug)]
pub struct SelectedPieceShadow;

#[derive(Component)]
pub struct Cursor {
    pub timer: Timer,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.15, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub struct Grid;

#[derive(Debug, Clone, Event)]
pub struct LoadLevelEvent;

#[derive(Debug, Clone, Event)]
pub struct SaveLevelEvent;

#[derive(Debug, Clone, Component)]
pub struct CurrentPieceText;

const GRID_OFF: Color = Color::Rgba {
    red: 0.01,
    green: 0.2,
    blue: 1.0,
    alpha: 0.1,
};

const GRID_HALF: Color = Color::Rgba {
    red: 0.0,
    green: 0.8,
    blue: 1.0,
    alpha: 0.2,
};

const GRID_ON: Color = Color::Rgba {
    red: 1.0,
    green: 0.5,
    blue: 0.2,
    alpha: 0.9,
};

pub fn setup(mut commands: Commands, qc: Query<Entity, With<ECamera>>) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }
    // 2D orthographic camera
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(400.0);
    commands.spawn(cam).insert(ECamera);
    info!("Editor camera setup");
}

pub fn cleanup(
    mut commands: Commands,
    qc: Query<Entity, With<ECamera>>,
    qg: Query<Entity, With<EditorUI>>,
    qgs: Query<Entity, With<EditorSprite>>,
) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }
    // Despawn editor UI if not used
    for gui in qg.iter() {
        commands.entity(gui).despawn_recursive();
    }
    // Despawn editor sprites if not used
    for gs in qgs.iter() {
        commands.entity(gs).despawn_recursive();
    }
    info!("Editor cleanup");
}

pub fn setup_ui(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    _fonts: Res<Assets<Font>>,
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
        .insert(EditorUI)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(20.0),
                        height: Val::Percent(100.0),
                        min_width: Val::Px(0.0),
                        min_height: Val::Px(16.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::FlexEnd,
                        flex_direction: FlexDirection::Column,
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
                            max_height: Val::Percent(20.0),
                            flex_shrink: 1.0,
                            flex_grow: 0.0,
                            ..default()
                        },
                        image: handles.images.title.clone().into(),
                        ..default()
                    });

                    parent.spawn(TextBundle::from_section(
                        "Level Editor",
                        TextStyle {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 38.0,
                            color: Color::AQUAMARINE,
                        },
                    ));
                    parent.spawn(TextBundle::from_section(
                        "(F2) Load Map",
                        TextStyle {
                            font: handles.fonts.londrina.w100_thin.clone(),
                            font_size: 24.0,
                            color: Color::Rgba {
                                red: 1.0,
                                green: 1.0,
                                blue: 1.0,
                                alpha: 0.2,
                            },
                        },
                    ));
                    parent.spawn(TextBundle::from_section(
                        "(F3) Save Map",
                        TextStyle {
                            font: handles.fonts.londrina.w100_thin.clone(),
                            font_size: 24.0,
                            color: Color::Rgba {
                                red: 1.0,
                                green: 1.0,
                                blue: 1.0,
                                alpha: 0.2,
                            },
                        },
                    ));
                    parent
                        .spawn(TextBundle::from_section(
                            "Current piece:",
                            TextStyle {
                                font: handles.fonts.londrina.w300_light.clone(),
                                font_size: 24.0,
                                color: Color::Rgba {
                                    red: 1.0,
                                    green: 1.0,
                                    blue: 1.0,
                                    alpha: 0.2,
                                },
                            },
                        ))
                        .insert(CurrentPieceText);
                    parent.spawn(NodeBundle {
                        style: Style {
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            width: Val::Percent(80.0),
                            height: Val::Auto,
                            ..default()
                        },

                        ..default()
                    });
                });
        });
    info!("Editor UI loaded");

    let tb = board::TileBuilder::new(&images, &handles, &mut materials1);

    for x1 in -64..=64 {
        for y1 in -64..=64 {
            commands
                .spawn(tb.tile(board::TileSprite::Grid))
                .insert(EditorSprite)
                .insert(board::Position::new_i64(x1, y1, 0))
                .insert(Grid);
        }
    }
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(EditorSprite)
        .insert(board::Position::new_i64(0, 0, 0))
        .insert(Cursor::default())
        .with_children(|parent| {
            parent
                .spawn(tb.tile_custom_into(
                    board::TileSprite::Grid,
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::Rgba {
                                red: 0.7,
                                green: 0.9,
                                blue: 1.0,
                                alpha: 0.1,
                            },
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(0.0, 0.0, 1.0),
                        ..Default::default()
                    },
                ))
                .insert(SelectedPiece {
                    piece: board::TileSprite::Grid,
                });
            parent
                .spawn(tb.tile_custom_into(
                    board::TileSprite::Grid,
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::Rgba {
                                red: 1.0,
                                green: 1.0,
                                blue: 1.0,
                                alpha: 0.6,
                            },
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(0.0, 0.0, -0.0001),
                        ..Default::default()
                    },
                ))
                .insert(SelectedPieceShadow);
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::Rgba {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 0.25,
                    },
                    ..Default::default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..Default::default()
            });
        });
    ev_load.send(LoadLevelEvent);
}

#[allow(clippy::too_many_arguments)]
pub fn keyboard(
    time: Res<Time>,
    app_state: Res<State<root::State>>,
    mut app_next_state: ResMut<NextState<root::State>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut camera: Query<&mut Transform, With<ECamera>>,
    mut cursor: Query<(&mut board::Position, &mut Cursor)>,
    mut ev_load: EventWriter<LoadLevelEvent>,
    mut ev_save: EventWriter<SaveLevelEvent>,
) {
    if *app_state.get() != root::State::Editor {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_next_state.set(root::State::MainMenu);
    }
    if keyboard_input.just_pressed(KeyCode::F2) {
        ev_load.send(LoadLevelEvent);
    }
    if keyboard_input.just_pressed(KeyCode::F3) {
        ev_save.send(SaveLevelEvent);
    }

    for mut transform in camera.iter_mut() {
        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.x += 6.0;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.x -= 6.0;
        }
        if keyboard_input.pressed(KeyCode::Up) {
            transform.translation.y += 6.0;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            transform.translation.y -= 6.0;
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
    if keyboard_input.any_just_pressed([KeyCode::D, KeyCode::A, KeyCode::W, KeyCode::S]) {
        for (mut position, mut cursor) in cursor.iter_mut() {
            cursor.timer.reset();
            cursor.timer.set_duration(Duration::from_secs_f32(0.15));
            if keyboard_input.just_pressed(KeyCode::D) {
                position.x += 1.0;
            }
            if keyboard_input.just_pressed(KeyCode::A) {
                position.x -= 1.0;
            }
            if keyboard_input.just_pressed(KeyCode::W) {
                position.y += 1.0;
            }
            if keyboard_input.just_pressed(KeyCode::S) {
                position.y -= 1.0;
            }
        }
    } else {
        for (mut position, mut cursor) in cursor.iter_mut() {
            cursor.timer.tick(time.delta());
            if cursor.timer.just_finished() {
                if keyboard_input.any_pressed([KeyCode::D, KeyCode::A, KeyCode::W, KeyCode::S]) {
                    cursor.timer.set_duration(Duration::from_secs_f32(0.05));
                }

                if keyboard_input.pressed(KeyCode::D) {
                    position.x += 1.0;
                }
                if keyboard_input.pressed(KeyCode::A) {
                    position.x -= 1.0;
                }
                if keyboard_input.pressed(KeyCode::W) {
                    position.y += 1.0;
                }
                if keyboard_input.pressed(KeyCode::S) {
                    position.y -= 1.0;
                }
            }
        }
    }
}

pub fn highlight_grid(
    cursor: Query<(&board::Position, &Cursor), Without<Grid>>,
    mut grid: Query<(&board::Position, &mut Sprite, &Grid), Without<Cursor>>,
) {
    for (cur_pos, _) in cursor.iter() {
        for (grid_pos, mut sprite, _) in grid.iter_mut() {
            let dist = cur_pos.distance(grid_pos);
            let alpha = Color::Rgba {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 2.0 / (dist + 2.0),
            };
            let color = if grid_pos == cur_pos {
                GRID_ON
            } else if grid_pos.same_xy(cur_pos) {
                GRID_HALF
            } else {
                GRID_OFF
            };
            sprite.color = color * alpha.as_rgba_f32();
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn piece_selector_input(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    handles: Res<root::GameAssets>,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut qs: Query<&mut SelectedPiece>,
    qc: Query<&board::Position, With<Cursor>>,
    qsp: Query<(Entity, &board::Position), (With<board::Tile>, Without<Cursor>)>,
    mut qspt: Query<&mut Text, With<CurrentPieceText>>,
    mut ev_bdr: EventWriter<BoardDataToRebuild>,
    mut bf: ResMut<board::BoardData>,
) {
    for mut sp in qs.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Q) {
            sp.piece = sp.piece.prev();
        }
        if keyboard_input.just_pressed(KeyCode::E) {
            sp.piece = sp.piece.next();
        }
        for mut text in qspt.iter_mut() {
            for sec in text.sections.iter_mut() {
                sec.value = format!("Selected Piece: {}", sp.piece.name());
            }
        }
        if keyboard_input.just_pressed(KeyCode::Delete) {
            for pos in qc.iter() {
                for (sp_entity, sp_pos) in qsp.iter() {
                    if sp_pos == pos {
                        let entity = commands.entity(sp_entity);
                        entity.despawn_recursive();
                        bf.tilemap.remove(&pos.to_board_position());
                    }
                }
            }
        }
        if keyboard_input.just_pressed(KeyCode::Space) {
            let tb = board::TileBuilder::new(&images, &handles, &mut materials1);
            for pos in qc.iter() {
                if sp.piece != TileSprite::Grid {
                    let tile = board::Tile {
                        sprite: sp.piece,
                        variant: TileVariant::Base,
                    };
                    let bpos = pos.to_board_position();
                    if let Some(tm) = bf.tilemap.get(&bpos) {
                        let v = tm
                            .iter()
                            .filter_map(|(k, v)| {
                                if tile.sprite == k.sprite {
                                    Some(*v)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        v.into_iter()
                            .for_each(|v| commands.entity(v).despawn_recursive())
                    }
                    let id = tb.spawn_tile(&mut commands, tile, *pos, EditorSprite, true);
                    bf.tilemap.entry(bpos).or_default().insert(tile, id);
                }
            }
            ev_bdr.send(BoardDataToRebuild {
                lighting: true,
                collision: false,
            });
        }
    }
}

pub fn selected_piece_display(
    images: Res<Assets<Image>>,
    handles: Res<root::GameAssets>,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
    mut qs: Query<(&mut Sprite, &mut Handle<Image>, &SelectedPiece), Without<SelectedPieceShadow>>,
    mut qss: Query<(&mut Sprite, &mut Handle<Image>), With<SelectedPieceShadow>>,
) {
    let tb = board::TileBuilder::new(&images, &handles, &mut materials1);
    for (mut sprite, mut texture, sp) in qs.iter_mut() {
        sprite.anchor = sp.piece.anchor(&tb);
        *texture = sp.piece.texture(&tb);
        for (mut sprite, mut texture) in qss.iter_mut() {
            sprite.anchor = sp.piece.anchor(&tb);
            *texture = sp.piece.texture(&tb);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn load_level(
    mut ev: EventReader<LoadLevelEvent>,
    mut commands: Commands,
    images: Res<Assets<Image>>,
    handles: Res<root::GameAssets>,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
    qgs: Query<Entity, With<board::Tile>>,
    mut ev_bdr: EventWriter<BoardDataToRebuild>,
    mut bf: ResMut<board::BoardData>,
) {
    for _ in ev.read() {
        commands.init_resource::<board::BoardData>();

        info!("Load Level");
        let json_u8 = std::fs::read("default_map.json").unwrap();
        let json = std::str::from_utf8(&json_u8).unwrap();
        let level = levelparse::Level::deserialize_json(json).unwrap();
        // Despawn tiles before loading the level
        for gs in qgs.iter() {
            commands.entity(gs).despawn_recursive();
        }
        let tb = board::TileBuilder::new(&images, &handles, &mut materials1);

        for tile in level.tiles.iter() {
            let pos: board::Position = tile.position.into();
            let tile = board::Tile {
                sprite: tile.sprite,
                variant: tile.variant,
            };
            let id = tb.spawn_tile(&mut commands, tile, pos, EditorSprite, true);
            bf.tilemap
                .entry(pos.to_board_position())
                .or_default()
                .insert(tile, id);
        }
        ev_bdr.send(BoardDataToRebuild {
            lighting: true,
            collision: false,
        });
    }
}

pub fn save_level(
    mut ev: EventReader<SaveLevelEvent>,
    qsp: Query<(&board::Position, &board::Tile)>,
) {
    for _ in ev.read() {
        info!("Save Level");
        let mut tiles: Vec<levelparse::Tile> = vec![];

        for (pos, tile) in qsp.iter() {
            info!("pos: {:?}, tile: {:?}", pos, tile);
            let tile = levelparse::Tile {
                position: pos.into(),
                sprite: tile.sprite,
                variant: tile.variant,
            };
            tiles.push(tile);
        }

        let level = levelparse::Level { tiles };
        let json = level.serialize_json().unwrap();
        std::fs::write("default_map.json", json).unwrap();
    }
}

pub fn compute_visibility(
    vf: &mut HashMap<BoardPosition, f32>,
    cf: &HashMap<BoardPosition, CollisionFieldData>,
    pos_start: &board::Position,
) {
    let mut queue = VecDeque::new();
    let start = pos_start.to_board_position();
    queue.push_front(start.clone());

    *vf.entry(start.clone()).or_default() = 1.0;

    while let Some(pos) = queue.pop_back() {
        let pds = pos.to_position().distance(pos_start);
        let src_f = vf.get(&pos).cloned().unwrap_or_default();
        if !cf.get(&pos).map(|c| c.free).unwrap_or_default() {
            // If the current position analyzed is not free (a wall or out of bounds)
            // then stop extending.
            continue;
        }
        // let neighbors = [pos.left(), pos.top(), pos.bottom(), pos.right()];
        let neighbors = pos.xy_neighbors(1);
        for npos in neighbors {
            if npos == pos {
                continue;
            }
            if cf.contains_key(&npos) {
                let npds = npos.to_position().distance(pos_start);
                let npref = npos.distance(&pos);
                let f = if npds < 1.5 {
                    1.0
                } else {
                    (((npds - pds) / npref - 0.25) / 0.99).clamp(0.0, 1.0)
                };
                let mut dst_f = src_f * f;
                if dst_f < 0.00001 {
                    continue;
                }
                if !vf.contains_key(&npos) {
                    queue.push_front(npos.clone());
                }
                dst_f /= 1.0 + ((npds - 1.5) / 10.0).clamp(0.0, 4.0);
                let entry = vf.entry(npos.clone()).or_insert(dst_f / 2.0);
                *entry = 1.0 - (1.0 - *entry) * (1.0 - dst_f);
            }
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn apply_lighting(
    mut qt: Query<
        (
            &board::TileColor,
            &board::Position,
            &mut Sprite,
            &board::Tile,
            Option<&Children>,
        ),
        Without<CustomMaterial1>,
    >,
    mut qt2: Query<(
        &board::TileColor,
        &board::Position,
        &Handle<CustomMaterial1>,
        &board::Tile,
    )>,
    materials1: ResMut<Assets<CustomMaterial1>>,
    mut qtc: Query<&mut Sprite, Without<board::Position>>,
    qc: Query<&board::Position, With<Cursor>>,
    qp: Query<(&board::Position, &game::PlayerSprite, &board::Direction), Without<Cursor>>,
    mut bf: ResMut<board::BoardData>,
    gc: Res<game::GameConfig>,
) {
    const GAMMA_EXP: f32 = 2.0;
    const CENTER_EXP: f32 = 2.3;
    const CENTER_EXP_GAMMA: f32 = 1.9;
    const EYE_SPEED: f32 = 0.5;
    let mut cursor_exp: f32 = 0.001;
    let mut exp_count: f32 = 0.001;
    for pos in qc.iter() {
        let cursor_pos = pos.to_board_position();
        for npos in cursor_pos.xy_neighbors(1) {
            if let Some(lf) = bf.light_field.get(&npos) {
                cursor_exp += lf.lux.powf(GAMMA_EXP);
                exp_count += lf.lux.powf(GAMMA_EXP) / lf.lux + 0.001;
            }
        }
    }
    let mut visibility_field = HashMap::<BoardPosition, f32>::new();
    let mut flashlights = vec![];
    const FLASHLIGHT_ON: bool = true;
    const FLASHLIGHT_POWER: f32 = 3.0;
    // FIXME: This function should not be in level editor
    // FIXME: We need to track the current player of the client (might not be id=1)
    for (pos, player, direction) in qp.iter() {
        if FLASHLIGHT_ON {
            flashlights.push((pos, direction));
        }

        if player.id != gc.player_id {
            continue;
        }

        let cursor_pos = pos.to_board_position();
        for npos in cursor_pos.xy_neighbors(1) {
            if let Some(lf) = bf.light_field.get(&npos) {
                cursor_exp += lf.lux.powf(GAMMA_EXP);
                exp_count += lf.lux.powf(GAMMA_EXP) / lf.lux + 0.001;
            }
        }
        compute_visibility(&mut visibility_field, &bf.collision_field, pos);
    }
    let current_pos = qc.iter().next().or(qp.iter().find_map(|(pos, p, _d)| {
        if p.id == gc.player_id {
            Some(pos)
        } else {
            qc.iter().next()
        }
    }));

    // dbg for flashlights:
    // dbg!(flashlights);
    /*if let Some(fl) = flashlights.first() {
        println!("fl: pos: {:?} dir: {:?}", fl.0, fl.1);
    }*/
    // --
    cursor_exp /= exp_count;
    cursor_exp = (cursor_exp / CENTER_EXP).powf(CENTER_EXP_GAMMA.recip()) * CENTER_EXP + 0.01;
    if FLASHLIGHT_ON {
        // account for the eye seeing the flashlight on.
        cursor_exp += FLASHLIGHT_POWER.sqrt() / 8.0;
    }

    let exp_f = ((cursor_exp) / bf.current_exposure) / bf.current_exposure_accel.powi(30);
    let max_acc = 1.05;
    bf.current_exposure_accel =
        (bf.current_exposure_accel * 1000.0 + exp_f * EYE_SPEED) / (EYE_SPEED + 1000.0);
    if bf.current_exposure_accel > max_acc {
        bf.current_exposure_accel = max_acc;
    } else if bf.current_exposure_accel.recip() > max_acc {
        bf.current_exposure_accel = max_acc.recip();
    }
    bf.current_exposure_accel = bf.current_exposure_accel.powf(0.99);
    bf.current_exposure *= bf.current_exposure_accel;
    let exposure = bf.current_exposure;
    for (tcolor, pos, mut sprite, tile, children) in qt.iter_mut() {
        let opacity = current_pos
            .map(|&pp| tile.occlusion_type().occludes(pp, *pos))
            .unwrap_or(1.0);
        let bpos = pos.to_board_position();
        let src_color = tcolor.color;
        let mut dst_color = if let Some(lf) = bf.light_field.get(&bpos) {
            let r: f32 = (bpos.mini_hash() - 0.4) / 50.0;
            let rel_lux = lf.lux / exposure;
            board::compute_color_exposure(rel_lux, r, board::DARK_GAMMA, src_color)
        } else {
            src_color
        };
        dst_color.set_a(opacity.clamp(0.6, 1.0));
        if let Some(children) = children {
            for &child in children.iter() {
                let mut c_sprite = qtc.get_mut(child).unwrap();
                c_sprite.color = dst_color;
            }
        }
        dst_color.set_a(opacity.clamp(0.2, 1.0));
        sprite.color = dst_color;
        // dbg!(&sprite);
    }

    const VSMALL_PRIME: usize = 13;
    const BIG_PRIME: usize = 95629;
    let mask: usize = rand::thread_rng().gen();
    let lf = &bf.light_field;
    let start = Instant::now();
    let materials1 = materials1.into_inner();
    let mut change_count = 0;
    for (n, (tcolor, pos, mat, tile)) in qt2.iter_mut().enumerate() {
        let min_threshold = (((n * BIG_PRIME) ^ mask) % VSMALL_PRIME) as f32 / 10.0;
        // dbg!(&mat);
        let mut opacity: f32 = 1.0;
        // opacity = current_pos
        //     .map(|&pp| tile.occlusion_type().occludes(pp, *pos))
        //     .unwrap_or(1.0)
        //     .max(0.5);
        let bpos = pos.to_board_position();
        let src_color = tcolor.color;

        opacity *= src_color.a();

        let bpos_tr = bpos.bottom();
        let bpos_bl = bpos.top();
        let bpos_br = bpos.right();
        let bpos_tl = bpos.left();

        const FL_STRENGTH: f32 = 5.0 * FLASHLIGHT_POWER; // flashlight strength
        const FL_MIN_DST: f32 = 7.0; // minimum distance for flashlight

        let fpos_gamma = |bpos: &BoardPosition| -> Option<f32> {
            let rpos = bpos.to_position();
            let mut lux_fl = 0.0; // lux from flashlight

            for (flpos, fldir) in flashlights.iter() {
                let pdist = flpos.distance(&rpos);
                let focus = (fldir.distance() - 4.0).max(1.0) / 20.0;
                let lpos = *flpos + *fldir / (100.0 / focus);
                let mut lpos = lpos.unrotate_by_dir(fldir);
                let mut rpos = rpos.unrotate_by_dir(fldir);
                rpos.x -= lpos.x;
                rpos.y -= lpos.y;
                lpos.x = 0.0;
                lpos.y = 0.0;
                if rpos.x > 0.0 {
                    rpos.x = fastapprox::faster::pow(rpos.x, 1.0 / focus.clamp(1.0, 1.1));
                    // rpos.x = rpos.x.powf(1.0 / focus.clamp(1.0, 10.0));
                    rpos.y /= rpos.x * (focus - 1.0).clamp(0.0, 10.0) / 30.0 + 1.0;
                }
                if rpos.x < 0.0 {
                    rpos.x = -fastapprox::faster::pow(-rpos.x, (focus / 5.0 + 1.0).clamp(1.0, 3.0));
                    // rpos.x = -(-rpos.x).powf((focus / 5.0 + 1.0).clamp(1.0, 3.0));
                    rpos.y *= -rpos.x * (focus - 1.0).clamp(0.0, 10.0) / 30.0 + 1.0;
                }
                let dist = lpos.distance(&rpos);
                lux_fl +=
                    FL_STRENGTH / (dist * dist + FL_MIN_DST) * (pdist / 5.0 + 0.6).clamp(0.0, 1.0);
            }

            lf.get(bpos).map(|lf| (lf.lux + lux_fl) / exposure)
        };

        let lux_c = fpos_gamma(&bpos).unwrap_or(1.0);
        let mut lux_tr = fpos_gamma(&bpos_tr).unwrap_or(lux_c);
        let mut lux_tl = fpos_gamma(&bpos_tl).unwrap_or(lux_c);
        let mut lux_br = fpos_gamma(&bpos_br).unwrap_or(lux_c);
        let mut lux_bl = fpos_gamma(&bpos_bl).unwrap_or(lux_c);

        match tile.occlusion_type() {
            board::OcclusionType::None => {}
            board::OcclusionType::XAxis => {
                lux_tl = lux_c;
                lux_br = lux_c;
            }
            board::OcclusionType::YAxis => {
                lux_tr = lux_c;
                lux_bl = lux_c;
            }
            board::OcclusionType::Both => {
                lux_tl = lux_c;
                lux_br = lux_c;
                lux_tr = lux_c;
                lux_bl = lux_c;
            }
        }

        let mut dst_color = {
            let r: f32 = (bpos.mini_hash() - 0.4) / 50.0;
            board::compute_color_exposure(lux_c, r, board::DARK_GAMMA, src_color)
        };
        dst_color.set_a(opacity.clamp(0.6, 1.0));

        opacity = opacity
            .min(visibility_field.get(&bpos).copied().unwrap_or_default() * 2.0)
            .min(1.0);
        // if let Some(children) = children {
        //     for &child in children.iter() {
        //         let mut c_sprite = qtc.get_mut(child).unwrap();
        //         c_sprite.color = dst_color;
        //     }
        // }

        let mut new_mat = materials1.get(mat).unwrap().clone();
        let orig_mat = new_mat.clone();
        let mut dst_color = src_color; // <- remove brightness calculation for main tile.
        let src_a = new_mat.data.color.a();
        let opacity = opacity.clamp(0.000, 1.0);
        const A_DELTA: f32 = 0.2;
        let new_a = if (src_a - opacity).abs() < A_DELTA {
            opacity
        } else {
            src_a - A_DELTA * (src_a - opacity).signum()
        };
        dst_color.set_a(new_a);
        // const A_SOFT: f32 = 1.0;
        // dst_color.set_a((opacity.clamp(0.000, 1.0) + src_a * A_SOFT) / (1.0 + A_SOFT));
        new_mat.data.color = dst_color;

        const SMOOTH_F: f32 = 1.0;
        let f_gamma = |lux: f32| {
            (fastapprox::faster::pow(lux, board::LIGHT_GAMMA)
                + fastapprox::faster::pow(lux, 1.0 / board::DARK_GAMMA))
                / 2.0
        };
        new_mat.data.gamma = (new_mat.data.gamma * SMOOTH_F + f_gamma(lux_c)) / (1.0 + SMOOTH_F);
        new_mat.data.gtl = (new_mat.data.gtl * SMOOTH_F + f_gamma(lux_tl)) / (1.0 + SMOOTH_F);
        new_mat.data.gtr = (new_mat.data.gtr * SMOOTH_F + f_gamma(lux_tr)) / (1.0 + SMOOTH_F);
        new_mat.data.gbl = (new_mat.data.gbl * SMOOTH_F + f_gamma(lux_bl)) / (1.0 + SMOOTH_F);
        new_mat.data.gbr = (new_mat.data.gbr * SMOOTH_F + f_gamma(lux_br)) / (1.0 + SMOOTH_F);

        let delta = orig_mat.data.delta(&new_mat.data);

        if delta > 0.02 + min_threshold {
            let mat = materials1.get_mut(mat).unwrap();
            mat.data = new_mat.data;
            change_count += 1;
        }
    }
    if mask % 255 == 0 {
        warn!("change_count: {}", &change_count);
        warn!("apply_lighting elapsed: {:?}", start.elapsed());
    }
}
