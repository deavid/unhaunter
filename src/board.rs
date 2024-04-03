use std::{f32::consts::PI, time::Duration};

use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    utils::{HashMap, HashSet, Instant},
};
use fastapprox::faster;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    behavior::{self, Behavior, SpriteCVOKey},
    ghost_definitions::Evidence,
    maplight,
    materials::CustomMaterial1,
};

#[derive(Component, Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub global_z: f32,
}

impl std::ops::Add<Direction> for &Position {
    type Output = Position;

    fn add(self, rhs: Direction) -> Self::Output {
        Position {
            x: self.x + rhs.dx,
            y: self.y + rhs.dy,
            z: self.z + rhs.dz,
            global_z: self.global_z,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Direction {
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

impl Direction {
    pub fn distance(&self) -> f32 {
        (self.dx.powi(2) + self.dy.powi(2) + self.dz.powi(2)).sqrt()
    }

    pub fn normalized(&self) -> Self {
        let dst = self.distance() + 0.000000001;
        Self {
            dx: self.dx / dst,
            dy: self.dy / dst,
            dz: self.dz / dst,
        }
    }
    pub fn to_screen_coord(self) -> Vec3 {
        let x =
            self.dx * PERSPECTIVE_X[0] + self.dy * PERSPECTIVE_Y[0] + self.dz * PERSPECTIVE_Z[0];
        let y =
            self.dx * PERSPECTIVE_X[1] + self.dy * PERSPECTIVE_Y[1] + self.dz * PERSPECTIVE_Z[1];
        let z =
            self.dx * PERSPECTIVE_X[2] + self.dy * PERSPECTIVE_Y[2] + self.dz * PERSPECTIVE_Z[2];
        Vec3::new(x, y, z)
    }
}

impl std::ops::Mul<f32> for &Direction {
    type Output = Direction;

    fn mul(self, rhs: f32) -> Self::Output {
        Direction {
            dx: self.dx * rhs,
            dy: self.dy * rhs,
            dz: self.dz * rhs,
        }
    }
}

impl std::ops::Div<f32> for &Direction {
    type Output = Direction;

    fn div(self, rhs: f32) -> Self::Output {
        Direction {
            dx: self.dx / rhs,
            dy: self.dy / rhs,
            dz: self.dz / rhs,
        }
    }
}

impl std::ops::Div<f32> for Direction {
    type Output = Direction;

    fn div(self, rhs: f32) -> Self::Output {
        Direction {
            dx: self.dx / rhs,
            dy: self.dy / rhs,
            dz: self.dz / rhs,
        }
    }
}

impl std::ops::Add<Direction> for Direction {
    type Output = Direction;

    fn add(self, rhs: Direction) -> Self::Output {
        Direction {
            dx: self.dx + rhs.dx,
            dy: self.dy + rhs.dy,
            dz: self.dz + rhs.dz,
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self {
            dx: 1.0,
            dy: 0.0,
            dz: 0.0,
        }
    }
}

const EPSILON: f32 = 0.0001;

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.same_x(other) && self.same_y(other) && self.same_z(other)
    }
}

// old perspective (9x20cm)
// const SUBTL: f32 = 9.0;

// new perspective (3x20cm)
const SUBTL: f32 = 3.0;

// new perspective (3x20cm) - reduced
// const SUBTL: f32 = 2.5;

const PERSPECTIVE_X: [f32; 3] = [4.0 * SUBTL, -2.0 * SUBTL, 0.0001];
const PERSPECTIVE_Y: [f32; 3] = [4.0 * SUBTL, 2.0 * SUBTL, -0.0001];
const PERSPECTIVE_Z: [f32; 3] = [0.0, 4.0 * 11.0, 0.01];

impl Position {
    pub fn new_i64(x: i64, y: i64, z: i64) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            z: z as f32,
            global_z: 0 as f32,
        }
    }
    pub fn into_global_z(mut self, global_z: f32) -> Self {
        self.global_z = global_z;
        self
    }
    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
    pub fn to_screen_coord(self) -> Vec3 {
        let x = self.x * PERSPECTIVE_X[0] + self.y * PERSPECTIVE_Y[0] + self.z * PERSPECTIVE_Z[0];
        let y = self.x * PERSPECTIVE_X[1] + self.y * PERSPECTIVE_Y[1] + self.z * PERSPECTIVE_Z[1];
        let z = self.x * PERSPECTIVE_X[2] + self.y * PERSPECTIVE_Y[2] + self.z * PERSPECTIVE_Z[2];
        Vec3::new(x, y, z + self.global_z)
    }
    pub fn same_x(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < EPSILON
    }
    pub fn same_y(&self, other: &Self) -> bool {
        (self.y - other.y).abs() < EPSILON
    }
    pub fn same_z(&self, other: &Self) -> bool {
        (self.z - other.z).abs() < EPSILON
    }
    pub fn same_xy(&self, other: &Self) -> bool {
        self.same_x(other) || self.same_y(other)
    }
    pub fn distance(&self, other: &Self) -> f32 {
        self.distance2(other).sqrt()
    }
    pub fn distance2(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }
    pub fn distance_taxicab(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx.abs() + dy.abs() + dz.abs()
    }
    pub fn to_board_position(self) -> BoardPosition {
        BoardPosition {
            x: self.x.round() as i64,
            y: self.y.round() as i64,
            z: self.z.round() as i64,
        }
    }
    pub fn rotate_by_dir(&self, dir: &Direction) -> Self {
        let dir = dir.normalized();
        // CAUTION: This is not possible with a single vector. Most likely wrong.
        let x_axis = Direction {
            dx: dir.dx,
            dy: dir.dy,
            dz: dir.dz,
        };
        let y_axis = Direction {
            dx: -dir.dy,
            dy: dir.dx,
            dz: dir.dz,
        };
        let z_axis = Direction {
            dx: -dir.dy,
            dy: dir.dz,
            dz: dir.dx,
        };

        Self {
            x: self.x * x_axis.dx + self.y * y_axis.dx + self.z * z_axis.dx,
            y: self.x * x_axis.dy + self.y * y_axis.dy + self.z * z_axis.dy,
            z: self.x * x_axis.dz + self.y * y_axis.dz + self.z * z_axis.dz,
            global_z: self.global_z,
        }
    }
    pub fn unrotate_by_dir(&self, dir: &Direction) -> Self {
        // ... probably wrong...
        let dir = Direction {
            dx: dir.dx,
            dy: -dir.dy,
            dz: -dir.dz,
        };
        self.rotate_by_dir(&dir)
    }
    pub fn delta(self, rhs: Position) -> Direction {
        Direction {
            dx: self.x - rhs.x,
            dy: self.y - rhs.y,
            dz: self.z - rhs.z,
        }
    }
}

impl std::ops::Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            global_z: self.global_z - rhs.global_z,
        }
    }
}

impl std::ops::Sub for &Position {
    type Output = Position;

    fn sub(self, rhs: Self) -> Self::Output {
        Position {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            global_z: self.global_z - rhs.global_z,
        }
    }
}
#[derive(Component, Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct BoardPosition {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl BoardPosition {
    pub fn to_position(&self) -> Position {
        Position {
            x: self.x as f32,
            y: self.y as f32,
            z: self.z as f32,
            global_z: 0.0,
        }
    }
    pub fn left(&self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y,
            z: self.z,
        }
    }
    pub fn right(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
            z: self.z,
        }
    }
    pub fn top(&self) -> Self {
        Self {
            x: self.x,
            y: self.y - 1,
            z: self.z,
        }
    }
    pub fn bottom(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
            z: self.z,
        }
    }
    pub fn xy_neighbors_buf(&self, dist: u32, out: &mut Vec<BoardPosition>) {
        out.clear();
        let dist = dist as i64;
        for x in -dist..=dist {
            for y in -dist..=dist {
                let pos = BoardPosition {
                    x: self.x + x,
                    y: self.y + y,
                    z: self.z,
                };
                out.push(pos);
            }
        }
    }
    pub fn xy_neighbors_buf_clamped(
        &self,
        dist: u32,
        out: &mut Vec<BoardPosition>,
        min_x: i64,
        max_x: i64,
        min_y: i64,
        max_y: i64,
    ) {
        out.clear();
        let dist = dist as i64;
        let x1 = (self.x - dist).clamp(min_x, max_x);
        let x2 = (self.x + dist).clamp(min_x, max_x);
        let y1 = (self.y - dist).clamp(min_y, max_y);
        let y2 = (self.y + dist).clamp(min_y, max_y);
        for x in x1..=x2 {
            for y in y1..=y2 {
                let pos = BoardPosition { x, y, z: self.z };
                out.push(pos);
            }
        }
    }
    pub fn xy_neighbors(&self, dist: u32) -> Vec<BoardPosition> {
        let mut ret: Vec<BoardPosition> = Vec::with_capacity((dist * dist * 4 + dist * 8) as usize);
        self.xy_neighbors_buf(dist, &mut ret);
        ret
    }
    pub fn distance(&self, other: &Self) -> f32 {
        let dx = self.x as f32 - other.x as f32;
        let dy = self.y as f32 - other.y as f32;
        let dz = self.z as f32 - other.z as f32;

        (dx.powi(2) + dy.powi(2) + dz.powi(2)).sqrt()
    }

    pub fn fast_distance_xy(&self, other: &Self) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        fastapprox::fast::pow(dx * dx + dy * dy, 0.5)
    }

    pub fn shadow_proximity(&self, shadow: &Self, tile: &Self) -> f32 {
        // This function assumes all points in the same Z plane.
        let sdx = self.x as f32 - shadow.x as f32;
        let sdy = self.y as f32 - shadow.y as f32;
        let sm = (sdx.powi(2) + sdy.powi(2)).sqrt();

        let tdx = self.x as f32 - tile.x as f32;
        let tdy = self.y as f32 - tile.y as f32;
        let tm = (tdx.powi(2) + tdy.powi(2)).sqrt();

        // Now convert tile vector into the same magnitude as the shadow vector:
        let tdx = tdx * sm / tm;
        let tdy = tdy * sm / tm;

        // The output of this function is the proximity scaled to the shadow point.
        // Where 0 .. 0.5 is full coverage, 1.0 is half coverage, and anything larger is no coverage.

        let dx = tdx - sdx;
        let dy = tdy - sdy;
        (dx.powi(2) + dy.powi(2)).sqrt()
    }
    pub fn mini_hash(&self) -> f32 {
        let h: i64 = ((self.x + 41) % 61 + (self.y * 13 + 47) % 67 + (self.z * 29 + 59) % 79) % 109;
        h as f32 / 109.0
    }
}

pub fn apply_perspective(mut q: Query<(&Position, &mut Transform)>) {
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = pos.to_screen_coord();
    }
}

#[derive(Clone)]
pub enum Bdl {
    Mmb(MaterialMesh2dBundle<CustomMaterial1>),
    Sb(SpriteBundle),
}
#[derive(Clone)]
pub struct MapTileComponents {
    pub bundle: Bdl,
    pub behavior: Behavior,
}

#[derive(Clone, Default, Resource)]
pub struct SpriteDB {
    pub map_tile: HashMap<(String, u32), MapTileComponents>,
    pub cvo_idx: HashMap<SpriteCVOKey, Vec<(String, u32)>>,
}

#[derive(Clone, Default, Resource)]
pub struct RoomDB {
    pub room_tiles: HashMap<BoardPosition, String>,
    pub room_state: HashMap<String, behavior::State>,
}

impl SpriteDB {
    pub fn clear(&mut self) {
        self.map_tile.clear();
        self.cvo_idx.clear();
    }
}

#[derive(Clone, Debug, Default, Event)]
pub struct BoardDataToRebuild {
    pub lighting: bool,
    pub collision: bool,
}

#[derive(Clone, Debug, Resource, Default)]
pub struct VisibilityData {
    pub visibility_field: HashMap<BoardPosition, f32>,
}

#[derive(Clone, Debug, Resource)]
pub struct BoardData {
    pub light_field: HashMap<BoardPosition, LightFieldData>,
    pub collision_field: HashMap<BoardPosition, CollisionFieldData>,
    pub temperature_field: HashMap<BoardPosition, f32>,
    pub sound_field: HashMap<BoardPosition, Vec<Vec2>>,
    pub breach_pos: Position,
    pub ambient_temp: f32,
    pub exposure_lux: f32,
    pub current_exposure: f32,
    pub current_exposure_accel: f32,
    pub evidences: HashSet<Evidence>,
}

impl FromWorld for BoardData {
    fn from_world(_world: &mut World) -> Self {
        // Using from_world to initialize is not needed but just in case we need it later.
        Self {
            collision_field: HashMap::new(),
            light_field: HashMap::new(),
            temperature_field: HashMap::new(),
            sound_field: HashMap::new(),
            exposure_lux: 1.0,
            current_exposure: 1.0,
            current_exposure_accel: 1.0,
            ambient_temp: 15.0,
            evidences: HashSet::new(),
            breach_pos: Position::new_i64(0, 0, 0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LightFieldData {
    pub lux: f32,
    pub transmissivity: f32,
    pub additional: maplight::LightData,
}

impl Default for LightFieldData {
    fn default() -> Self {
        Self {
            lux: 0.0,
            transmissivity: 1.0,
            additional: maplight::LightData::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Copy)]
pub struct CollisionFieldData {
    pub player_free: bool,
    pub ghost_free: bool,
    pub see_through: bool,
}

#[derive(Clone, Debug)]
pub struct LightFieldSector {
    // field: Vec<Vec<Vec<Option<Box<LightFieldData>>>>>,
    field: Vec<LightFieldData>,
    min_x: i64,
    min_y: i64,
    _min_z: i64,
    sz_x: usize,
    sz_y: usize,
    _sz_z: usize,
}
// FIXME: This has exactly the same computation as HashMap, at least for the part that it matters.
impl LightFieldSector {
    pub fn new(min_x: i64, min_y: i64, min_z: i64, max_x: i64, max_y: i64, max_z: i64) -> Self {
        let sz_x = (max_x - min_x + 1).max(0) as usize;
        let sz_y = (max_y - min_y + 1).max(0) as usize;
        let sz_z = (max_z - min_z + 1).max(0) as usize;
        let light_field: Vec<LightFieldData> =
            vec![LightFieldData::default(); sz_x * sz_y * sz_z + 15000];
        Self {
            field: light_field,
            min_x,
            min_y,
            _min_z: min_z,
            sz_x,
            sz_y,
            _sz_z: sz_z,
        }
    }

    #[inline]
    fn vec_coord(&self, x: i64, y: i64, _z: i64) -> usize {
        let x = x - self.min_x;
        let y = y - self.min_y;
        // let z = z - self.min_z;
        // These are purposefully allowing overflow and clamping to an out of bounds value.
        let x = (x as usize).min(self.sz_x);
        let y = (y as usize).min(self.sz_y);
        // let z = (z as usize).min(self.sz_z);

        x + y * self.sz_x // + z * self.sz_x * self.sz_y

        // (x & 0xF) | ((y & 0xF) << 4) | ((x & 0xFFFFF0) << 4) | ((y & 0xFFFFF0) << 8)
    }

    pub fn get_mut(&mut self, x: i64, y: i64, z: i64) -> Option<&mut LightFieldData> {
        let xyz = self.vec_coord(x, y, z);
        self.field.get_mut(xyz)
    }
    pub fn get_pos(&self, p: &BoardPosition) -> Option<&LightFieldData> {
        self.get(p.x, p.y, p.z)
    }
    pub fn get_mut_pos(&mut self, p: &BoardPosition) -> Option<&mut LightFieldData> {
        self.get_mut(p.x, p.y, p.z)
    }

    #[inline]
    pub fn get(&self, x: i64, y: i64, z: i64) -> Option<&LightFieldData> {
        let xyz = self.vec_coord(x, y, z);
        self.field.get(xyz)
    }

    /// get_pos_unchecked: Does not seem to be any faster.
    // #[inline]
    // pub unsafe fn get_pos_unchecked(&self, p: &BoardPosition) -> &LightFieldData {
    //     // let xyz = self.vec_coord(p.x, p.y, p.z);
    //     let xyz = (p.x - self.min_x) as usize + (p.y - self.min_y) as usize * self.sz_x;
    //     self.field.get_unchecked(xyz)
    // }

    pub fn insert(&mut self, x: i64, y: i64, z: i64, lfd: LightFieldData) {
        let xyz = self.vec_coord(x, y, z);
        self.field[xyz] = lfd;
    }
}

#[derive(Debug, Clone)]
struct CachedBoardPos {
    dist: [[f32; Self::SZ]; Self::SZ],
    angle: [[usize; Self::SZ]; Self::SZ],
    angle_range: [[(i64, i64); Self::SZ]; Self::SZ],
}

impl CachedBoardPos {
    const CENTER: i64 = 32;
    const SZ: usize = (Self::CENTER * 2 + 1) as usize;
    /// Perimeter of the circle for indexing.
    const TAU_I: usize = 48 * 2;

    fn new() -> Self {
        let mut r = Self {
            dist: [[0.0; Self::SZ]; Self::SZ],
            angle: [[0; Self::SZ]; Self::SZ],
            angle_range: [[(0, 0); Self::SZ]; Self::SZ],
        };
        r.compute_angle();
        r.compute_dist();
        r
    }

    fn compute_dist(&mut self) {
        for (x, xv) in self.dist.iter_mut().enumerate() {
            for (y, yv) in xv.iter_mut().enumerate() {
                let x: f32 = x as f32 - Self::CENTER as f32;
                let y: f32 = y as f32 - Self::CENTER as f32;
                let dist: f32 = (x * x + y * y).sqrt();
                *yv = dist;
            }
        }
    }
    fn compute_angle(&mut self) {
        for (x, xv) in self.angle.iter_mut().enumerate() {
            for (y, yv) in xv.iter_mut().enumerate() {
                let x: f32 = x as f32 - Self::CENTER as f32;
                let y: f32 = y as f32 - Self::CENTER as f32;
                let dist: f32 = (x * x + y * y).sqrt();

                let x = x / dist;
                let y = y / dist;

                let angle = x.acos() * y.signum() * Self::TAU_I as f32 / PI / 2.0;
                let angle_i = (angle.round() as i64).rem_euclid(Self::TAU_I as i64);
                *yv = angle_i as usize;
            }
        }
        for y in Self::CENTER - 3..=Self::CENTER + 3 {
            let mut v: Vec<usize> = vec![];
            for x in Self::CENTER - 3..=Self::CENTER + 3 {
                v.push(self.angle[x as usize][y as usize]);
            }
        }

        for (x, xv) in self.angle_range.iter_mut().enumerate() {
            for (y, yv) in xv.iter_mut().enumerate() {
                let orig_angle = self.angle[x][y];
                // if angle < Self::TAU_I / 4 || angle > Self::TAU_I - Self::TAU_I / 4 {
                //     // Angles closer to zero need correction to avoid looking on the wrong place

                // }
                let mut min_angle: i64 = 0;
                let mut max_angle: i64 = 0;
                let x: f32 = x as f32 - Self::CENTER as f32;
                let y: f32 = y as f32 - Self::CENTER as f32;
                for x1 in [x - 0.5, x + 0.5] {
                    for y1 in [y - 0.5, y + 0.5] {
                        let dist: f32 = (x1 * x1 + y1 * y1).sqrt();
                        let x1 = x1 / dist;
                        let y1 = y1 / dist;
                        let angle = x1.acos() * y1.signum() * Self::TAU_I as f32 / PI / 2.0;
                        let mut angle_i = angle.round() as i64 - orig_angle as i64;
                        if angle_i.abs() > Self::TAU_I as i64 / 2 {
                            angle_i -= Self::TAU_I as i64 * angle_i.signum();
                        }
                        min_angle = min_angle.min(angle_i);
                        max_angle = max_angle.max(angle_i);
                    }
                }
                *yv = (min_angle, max_angle);
            }
        }
        for y in Self::CENTER - 3..=Self::CENTER + 3 {
            let mut v: Vec<(i64, i64)> = vec![];
            for x in Self::CENTER - 3..=Self::CENTER + 3 {
                v.push(self.angle_range[x as usize][y as usize]);
            }
        }
    }
    fn bpos_dist(&self, s: &BoardPosition, d: &BoardPosition) -> f32 {
        let x = (d.x - s.x + Self::CENTER) as usize;
        let y = (d.y - s.y + Self::CENTER) as usize;
        // self.dist[x][y]
        unsafe { *self.dist.get_unchecked(x).get_unchecked(y) }
    }
    fn bpos_angle(&self, s: &BoardPosition, d: &BoardPosition) -> usize {
        let x = (d.x - s.x + Self::CENTER) as usize;
        let y = (d.y - s.y + Self::CENTER) as usize;
        // self.angle[x][y]
        unsafe { *self.angle.get_unchecked(x).get_unchecked(y) }
    }
    fn bpos_angle_range(&self, s: &BoardPosition, d: &BoardPosition) -> (i64, i64) {
        let x = (d.x - s.x + Self::CENTER) as usize;
        let y = (d.y - s.y + Self::CENTER) as usize;
        // self.angle_range[x][y]
        unsafe { *self.angle_range.get_unchecked(x).get_unchecked(y) }
    }
}

pub fn boardfield_update(
    mut bf: ResMut<BoardData>,
    mut ev_bdr: EventReader<BoardDataToRebuild>,

    qt: Query<(&Position, &Behavior)>,
) {
    let mut rng = rand::thread_rng();
    // Here we will recreate the field (if needed? - not sure how to detect that)
    // ... maybe add a timer since last update.
    for bfr in ev_bdr.read() {
        if bfr.collision {
            // info!("Collision rebuild");
            bf.collision_field.clear();
            for (pos, _behavior) in qt.iter().filter(|(_p, b)| b.p.movement.walkable) {
                let pos = pos.to_board_position();
                let colfd = CollisionFieldData {
                    player_free: true,
                    ghost_free: true,
                    see_through: false,
                };
                bf.collision_field.insert(pos, colfd);
            }
            for (pos, behavior) in qt.iter().filter(|(_p, b)| b.p.movement.player_collision) {
                let pos = pos.to_board_position();
                let colfd = CollisionFieldData {
                    player_free: false,
                    ghost_free: !behavior.p.movement.ghost_collision,
                    see_through: behavior.p.light.see_through,
                };
                bf.collision_field.insert(pos, colfd);
            }
        }
        // Create temperature field - only missing data
        let valid_k: Vec<_> = bf.collision_field.keys().cloned().collect();
        let ambient_temp = bf.ambient_temp;
        let mut added_temps: Vec<BoardPosition> = vec![];
        // Randomize initial temperatures so the player cannot exploit the fact that the data is "flat" at the beginning
        for pos in valid_k.into_iter() {
            let missing = bf.temperature_field.get(&pos).is_none();
            if missing {
                let ambient = ambient_temp + rng.gen_range(-10.0..10.0);
                added_temps.push(pos.clone());
                bf.temperature_field.insert(pos, ambient);
            }
        }
        // Smoothen after first initialization so it is not as jumpy.
        for _ in 0..16 {
            for pos in added_temps.iter() {
                let nbors = pos.xy_neighbors(1);
                let mut t_temp = 0.0;
                let mut count = 0.0;
                let free_tot = bf
                    .collision_field
                    .get(pos)
                    .map(|x| x.player_free)
                    .unwrap_or(true);
                for npos in &nbors {
                    let free = bf
                        .collision_field
                        .get(npos)
                        .map(|x| x.player_free)
                        .unwrap_or(true);
                    if free {
                        t_temp += bf
                            .temperature_field
                            .get(npos)
                            .copied()
                            .unwrap_or(ambient_temp);
                        count += 1.0;
                    }
                }
                if free_tot {
                    t_temp /= count;
                    bf.temperature_field
                        .entry(pos.clone())
                        .and_modify(|x| *x = t_temp);
                }
            }
        }

        if bfr.lighting {
            // Rebuild lighting field since it has changed
            // info!("Lighting rebuild");
            let build_start_time = Instant::now();
            let cbp = CachedBoardPos::new();

            bf.exposure_lux = 1.0;
            bf.light_field.clear();
            // Dividing by 4 so later we don't get an overflow if there's no map.
            let first_p = qt
                .iter()
                .next()
                .map(|(p, _b)| p.to_board_position())
                .unwrap_or_default();
            let mut min_x = first_p.x;
            let mut min_y = first_p.y;
            let mut min_z = first_p.z;
            let mut max_x = first_p.x;
            let mut max_y = first_p.y;
            let mut max_z = first_p.z;
            for (pos, behavior) in qt.iter() {
                let pos = pos.to_board_position();
                min_x = min_x.min(pos.x);
                min_y = min_y.min(pos.y);
                min_z = min_z.min(pos.z);
                max_x = max_x.max(pos.x);
                max_y = max_y.max(pos.y);
                max_z = max_z.max(pos.z);
                let src = bf.light_field.get(&pos).cloned().unwrap_or(LightFieldData {
                    lux: 0.0,
                    transmissivity: 1.0,
                    additional: maplight::LightData::default(),
                });
                let lightdata = LightFieldData {
                    lux: behavior.p.light.emmisivity_lumens() + src.lux,
                    transmissivity: behavior.p.light.transmissivity_factor() * src.transmissivity
                        + 0.0001,
                    additional: src.additional.add(&behavior.p.light.additional_data()),
                };
                bf.light_field.insert(pos, lightdata);
            }
            // info!(
            //     "Collecting time: {:?} - sz: {}",
            //     build_start_time.elapsed(),
            //     bf.light_field.len()
            // );
            let mut lfs = LightFieldSector::new(min_x, min_y, min_z, max_x, max_y, max_z);
            for (k, v) in bf.light_field.iter() {
                lfs.insert(k.x, k.y, k.z, v.clone());
            }

            let mut nbors_buf = Vec::with_capacity(52 * 52);

            let mut lfs_clone_time_total = Duration::ZERO;
            let mut shadows_time_total = Duration::ZERO;
            let mut store_lfs_time_total = Duration::ZERO;
            for step in 0..3 {
                let lfs_clone_time = Instant::now();
                let src_lfs = lfs.clone();
                lfs_clone_time_total += lfs_clone_time.elapsed();
                let size = match step {
                    0 => 24,
                    1 => 12,
                    2 => 1,
                    _ => 6,
                };
                for x in min_x..=max_x {
                    for y in min_y..=max_y {
                        for z in min_z..=max_z {
                            let Some(src) = src_lfs.get(x, y, z) else {
                                continue;
                            };
                            if src.transmissivity < 0.5 && step > 0 {
                                // Reduce light spread through walls
                                // FIXME: If the light is on the wall, this breaks (and this is possible since the wall is really 1/3rd of the tile)
                                continue;
                            }
                            let root_pos = BoardPosition { x, y, z };

                            let mut src_lux = src.lux;
                            let min_lux = match step {
                                0 => 0.001,
                                1 => 0.000001,
                                _ => 0.0000000001,
                            };
                            let max_lux = match step {
                                0 => f32::MAX,
                                1 => 10000.0,
                                2 => 1000.0,
                                3 => 0.1,
                                _ => 0.01,
                            };
                            if src_lux < min_lux {
                                continue;
                            }
                            if src_lux > max_lux {
                                continue;
                            }
                            // Optimize next steps by only looking to harsh differences.
                            root_pos.xy_neighbors_buf_clamped(
                                1,
                                &mut nbors_buf,
                                min_x,
                                max_x,
                                min_y,
                                max_y,
                            );
                            let nbors = &nbors_buf;

                            let min_lux = nbors
                                .iter()
                                .filter_map(|b| {
                                    lfs.get_pos(b).map(|l| ordered_float::OrderedFloat(l.lux))
                                })
                                .min()
                                .unwrap_or_default()
                                .into_inner()
                                + 0.0000001;
                            if src_lux / min_lux < 1.05 {
                                // If there's less than 10% difference, this does not need to be re-examined.
                                continue;
                            }
                            // This controls how harsh is the light
                            if step > 0 {
                                src_lux /= 1.5;
                            } else {
                                src_lux /= 1.01;
                            }

                            let shadows_time = Instant::now();
                            // This takes time to process:
                            root_pos.xy_neighbors_buf_clamped(
                                size,
                                &mut nbors_buf,
                                min_x,
                                max_x,
                                min_y,
                                max_y,
                            );
                            let nbors = &nbors_buf;

                            // reset the light value for this light, so we don't count double.
                            lfs.get_mut_pos(&root_pos).unwrap().lux -= src_lux;
                            let mut shadow_dist = [(size + 1) as f32; CachedBoardPos::TAU_I];
                            // Compute shadows
                            for pillar_pos in nbors.iter() {
                                // 60% of the time spent in compute shadows is obtaining this:
                                let Some(lf) = lfs.get_pos(pillar_pos) else {
                                    continue;
                                };

                                // let lf = unsafe { lfs.get_pos_unchecked(pillar_pos) };
                                // t_x += lf.lux;
                                // continue;
                                let consider_opaque = lf.transmissivity < 0.5;
                                if !consider_opaque {
                                    continue;
                                }
                                let min_dist = cbp.bpos_dist(&root_pos, pillar_pos);
                                let angle = cbp.bpos_angle(&root_pos, pillar_pos);
                                let angle_range = cbp.bpos_angle_range(&root_pos, pillar_pos);
                                for d in angle_range.0..=angle_range.1 {
                                    let ang = (angle as i64 + d)
                                        .rem_euclid(CachedBoardPos::TAU_I as i64)
                                        as usize;
                                    shadow_dist[ang] = shadow_dist[ang].min(min_dist);
                                }
                            }

                            shadows_time_total += shadows_time.elapsed();
                            // FIXME: Possibly we want to smooth shadow_dist here - a convolution with a gaussian or similar
                            // where we preserve the high values but smooth the transition to low ones.

                            if src.transmissivity < 0.5 {
                                // Reduce light spread through walls
                                shadow_dist.iter_mut().for_each(|x| *x = 0.0);
                            }
                            // let size = shadow_dist
                            //     .iter()
                            //     .map(|d| (d + 1.5).round() as u32)
                            //     .max()
                            //     .unwrap()
                            //     .min(size);
                            // let nbors = root_pos.xy_neighbors(size);
                            let light_height = 4.0;
                            // let mut total_lux = 0.1;
                            // for neighbor in nbors.iter() {
                            //     let dist = cbp.bpos_dist(&root_pos, neighbor);
                            //     let dist2 = dist + light_height;
                            //     let angle = cbp.bpos_angle(&root_pos, neighbor);
                            //     let sd = shadow_dist[angle];
                            //     let f = (faster::tanh(sd - dist - 0.5) + 1.0) / 2.0;
                            //     total_lux += f / dist2 / dist2;
                            // }
                            let store_lfs_time = Instant::now();

                            let total_lux = 2.0;
                            // new shadow method
                            for neighbor in nbors.iter() {
                                let dist = cbp.bpos_dist(&root_pos, neighbor);
                                // let dist = root_pos.fast_distance_xy(neighbor);
                                let dist2 = dist + light_height;
                                let angle = cbp.bpos_angle(&root_pos, neighbor);
                                let sd = shadow_dist[angle];
                                let lux_add = src_lux / dist2 / dist2 / total_lux;
                                if dist - 3.0 < sd {
                                    // FIXME: f here controls the bleed through walls.
                                    if let Some(lf) = lfs.get_mut_pos(neighbor) {
                                        const BLEED_TILES: f32 = 0.3;
                                        let f =
                                            (faster::tanh((sd - dist - 0.5) * BLEED_TILES.recip())
                                                + 1.0)
                                                / 2.0;
                                        // let f = 1.0;
                                        lf.lux += lux_add * f;
                                    }
                                }
                            }
                            store_lfs_time_total += store_lfs_time.elapsed();
                        }
                    }
                }
                // info!(
                //     "Light step {}: {:?}; per size: {:?}",
                //     step,
                //     step_time.elapsed(),
                //     step_time.elapsed() / size
                // );
            }
            for (k, v) in bf.light_field.iter_mut() {
                v.lux = lfs.get_pos(k).unwrap().lux;
            }

            // let's get an average of lux values
            let mut total_lux = 0.0;
            for (_, v) in bf.light_field.iter() {
                total_lux += v.lux;
            }
            let avg_lux = total_lux / bf.light_field.len() as f32;
            bf.exposure_lux = (avg_lux + 2.0) / 2.0;

            dbg!(lfs_clone_time_total);
            dbg!(shadows_time_total);
            dbg!(store_lfs_time_total);

            info!(
                "Lighting rebuild - complete: {:?}",
                build_start_time.elapsed()
            );
        }
    }
}

pub const DARK_GAMMA: f32 = 1.0;
pub const LIGHT_GAMMA: f32 = 1.1;

// pub const DARK_GAMMA: f32 = 1.5;
// pub const LIGHT_GAMMA: f32 = 2.5;

pub fn compute_color_exposure(
    rel_exposure: f32,
    dither: f32,
    gamma: f32,
    src_color: Color,
) -> Color {
    let exp = rel_exposure.powf(gamma.recip()) + dither;
    let dst_color: Color = if exp < 1.0 {
        Color::Rgba {
            red: src_color.r() * exp,
            green: src_color.g() * exp,
            blue: src_color.b() * exp,
            alpha: src_color.a(),
        }
    } else {
        let rexp = exp.recip();
        Color::Rgba {
            red: 1.0 - ((1.0 - src_color.r()) * rexp),
            green: 1.0 - ((1.0 - src_color.g()) * rexp),
            blue: 1.0 - ((1.0 - src_color.b()) * rexp),
            alpha: src_color.a(),
        }
    };
    dst_color
}

pub fn app_setup(app: &mut App) {
    app.init_resource::<BoardData>()
        .init_resource::<VisibilityData>()
        .init_resource::<SpriteDB>()
        .init_resource::<RoomDB>()
        .add_systems(Update, apply_perspective)
        .add_systems(PostUpdate, boardfield_update)
        .add_event::<BoardDataToRebuild>();
}
