use crate::behavior::component::{Interactive, RoomState};
use crate::behavior::Behavior;
use crate::board::{self, Bdl, BoardPosition, Position};
use crate::game::{InteractionExecutionType, RoomChangedEvent};
use crate::root;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Component, Debug)]
pub struct PlayerSprite {
    pub id: usize,
    pub controls: ControlKeys,
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

    /// Interaction key (open doors, switches, etc).
    pub activate: KeyCode,
    /// Grab stuff from the ground.
    pub grab: KeyCode,
    /// Drop stuff to the ground.
    pub drop: KeyCode,
    /// Trigger the left-hand item.
    pub torch: KeyCode,
    /// Trigger the right-hand item.
    pub trigger: KeyCode,
    /// Cycle through the items on the inventory.
    pub cycle: KeyCode,
    /// Swap the left hand item with the right hand one.
    pub swap: KeyCode,
}

impl ControlKeys {
    pub const WASD: Self = ControlKeys {
        up: KeyCode::W,
        down: KeyCode::S,
        left: KeyCode::A,
        right: KeyCode::D,
        activate: KeyCode::E,
        trigger: KeyCode::R,
        torch: KeyCode::T,
        cycle: KeyCode::Q,
        swap: KeyCode::Tab,
        drop: KeyCode::G,
        grab: KeyCode::F,
    };
    pub const IJKL: Self = ControlKeys {
        up: KeyCode::I,
        down: KeyCode::K,
        left: KeyCode::J,
        right: KeyCode::L,
        activate: KeyCode::O,
        torch: KeyCode::T,
        cycle: KeyCode::Unlabeled,
        swap: KeyCode::Unlabeled,
        grab: KeyCode::Unlabeled,
        drop: KeyCode::Unlabeled,
        trigger: KeyCode::Unlabeled,
    };
    pub const NONE: Self = ControlKeys {
        up: KeyCode::Unlabeled,
        down: KeyCode::Unlabeled,
        left: KeyCode::Unlabeled,
        right: KeyCode::Unlabeled,
        activate: KeyCode::Unlabeled,
        torch: KeyCode::Unlabeled,
        cycle: KeyCode::Unlabeled,
        swap: KeyCode::Unlabeled,
        grab: KeyCode::Unlabeled,
        drop: KeyCode::Unlabeled,
        trigger: KeyCode::Unlabeled,
    };
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn keyboard_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut players: Query<(
        &mut board::Position,
        &mut board::Direction,
        &mut PlayerSprite,
        &mut AnimationTimer,
    )>,
    colhand: CollisionHandler,
    interactables: Query<
        (
            Entity,
            &board::Position,
            &Interactive,
            &Behavior,
            Option<&RoomState>,
        ),
        Without<PlayerSprite>,
    >,
    mut interactive_stuff: InteractiveStuff,
    mut ev_room: EventWriter<RoomChangedEvent>,
) {
    const PLAYER_SPEED: f32 = 0.04;
    const DIR_MIN: f32 = 5.0;
    const DIR_MAX: f32 = 80.0;
    const DIR_STEPS: f32 = 15.0;
    const DIR_MAG2: f32 = DIR_MAX / DIR_STEPS;
    const DIR_RED: f32 = 1.001;
    for (mut pos, mut dir, player, mut anim) in players.iter_mut() {
        let col_delta = colhand.delta(&pos);
        pos.x -= col_delta.x;
        pos.y -= col_delta.y;

        let mut d = board::Direction {
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
            for (entity, item_pos, interactive, behavior, _) in interactables.iter() {
                let cp_delta = interactive.control_point_delta(behavior);
                // let old_dist = pos.delta(*item_pos);
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
                // if dist < 1.5 {
                //     dbg!(cp_delta, old_dist, new_dist, dref, dist);
                // }
                if dist < max_dist {
                    max_dist = dist + 0.00001;
                    selected_entity = Some(entity);
                }
            }
            if let Some(entity) = selected_entity {
                for (entity, item_pos, interactive, behavior, rs) in
                    interactables.iter().filter(|(e, _, _, _, _)| *e == entity)
                {
                    if interactive_stuff.execute_interaction(
                        entity,
                        item_pos,
                        Some(interactive),
                        behavior,
                        rs,
                        InteractionExecutionType::ChangeState,
                    ) {
                        ev_room.send(RoomChangedEvent);
                    }
                }
            }
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
    pub state: CharacterAnimationState,
    pub dir: CharacterAnimationDirection,
}

impl CharacterAnimation {
    pub fn from_dir(dx: f32, dy: f32) -> Self {
        let dst = (dx * dx + dy * dy).sqrt() + 0.001;
        let state = if dst > 1.0 {
            CharacterAnimationState::Walking
        } else {
            CharacterAnimationState::Standing
        };
        let dir = CharacterAnimationDirection::from_dir(dx, dy);
        Self { state, dir }
    }
    pub fn to_vec(self) -> Vec<usize> {
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
                let rel_lux = lf.lux / bf.current_exposure + 0.1;
                n_rel_lux += f;
                tot_rel_lux += rel_lux * f;
            }
        }
        let rel_lux = tot_rel_lux / n_rel_lux;
        tas.color = board::compute_color_exposure(rel_lux, 0.0, board::DARK_GAMMA, color);
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
            if !cf.player_free && Self::ENABLE_COLLISION {
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

#[derive(SystemParam)]
pub struct InteractiveStuff<'w, 's> {
    bf: Res<'w, board::SpriteDB>,
    commands: Commands<'w, 's>,
    materials1: ResMut<'w, Assets<crate::materials::CustomMaterial1>>,
    asset_server: Res<'w, AssetServer>,
    roomdb: ResMut<'w, board::RoomDB>,
    game_next_state: ResMut<'w, NextState<root::GameState>>,
}

impl<'w, 's> InteractiveStuff<'w, 's> {
    pub fn execute_interaction(
        &mut self,
        entity: Entity,
        item_pos: &Position,
        interactive: Option<&Interactive>,
        behavior: &Behavior,
        room_state: Option<&RoomState>,
        ietype: InteractionExecutionType,
    ) -> bool {
        let item_bpos = item_pos.to_board_position();
        let tuid = behavior.key_tuid();
        let cvo = behavior.key_cvo();
        if behavior.is_van_entry() {
            if ietype != InteractionExecutionType::ChangeState {
                return false;
            }
            if let Some(interactive) = interactive {
                let sound_file = interactive.sound_for_moving_into_state(behavior);
                self.commands.spawn(AudioBundle {
                    source: self.asset_server.load(sound_file),
                    settings: PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Despawn,
                        volume: bevy::audio::Volume::Relative(bevy::audio::VolumeLevel::new(1.0)),
                        speed: 1.0,
                        paused: false,
                        spatial: false,
                    },
                });
            }
            self.game_next_state.set(root::GameState::Truck);
        }
        for other_tuid in self.bf.cvo_idx.get(&cvo).unwrap().iter() {
            if *other_tuid == tuid {
                continue;
            }
            let mut e_commands = self.commands.get_entity(entity).unwrap();
            let other = self.bf.map_tile.get(other_tuid).unwrap();

            let mut beh = other.behavior.clone();
            beh.flip(behavior.p.flip);

            // In case it is connected to a room, we need to change room state.
            if let Some(room_state) = room_state {
                let item_roombpos = BoardPosition {
                    x: item_bpos.x + room_state.room_delta.x,
                    y: item_bpos.y + room_state.room_delta.y,
                    z: item_bpos.z + room_state.room_delta.z,
                };
                let room_name = self
                    .roomdb
                    .room_tiles
                    .get(&item_roombpos)
                    .cloned()
                    .unwrap_or_default();
                // dbg!(&room_state, &item_roombpos);
                // dbg!(&room_name);
                match ietype {
                    InteractionExecutionType::ChangeState => {
                        if let Some(main_room_state) = self.roomdb.room_state.get_mut(&room_name) {
                            *main_room_state = beh.state();
                        }
                    }
                    InteractionExecutionType::ReadRoomState => {
                        if let Some(main_room_state) = self.roomdb.room_state.get(&room_name) {
                            if *main_room_state != beh.state() {
                                continue;
                            }
                        }
                    }
                }
            }

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

            e_commands.insert(beh);
            if ietype == InteractionExecutionType::ChangeState {
                if let Some(interactive) = interactive {
                    let sound_file = interactive.sound_for_moving_into_state(&other.behavior);
                    self.commands.spawn(AudioBundle {
                        source: self.asset_server.load(sound_file),
                        settings: PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            volume: bevy::audio::Volume::Relative(bevy::audio::VolumeLevel::new(
                                1.0,
                            )),
                            speed: 1.0,
                            paused: false,
                            spatial: false,
                        },
                    });
                }
            }

            return true;
        }
        false
    }
}

pub fn app_setup(app: &mut App) {
    app.add_event::<RoomChangedEvent>()
        .add_systems(
            Update,
            keyboard_player.run_if(in_state(root::GameState::None)),
        )
        .add_systems(Update, animate_sprite)
        .add_systems(Update, player_coloring);
}