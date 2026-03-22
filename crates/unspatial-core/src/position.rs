use bevy::ecs::entity::{EntityMapper, MapEntities};
use bevy::prelude::*;
use bevy_math::Vec3;
use serde::{Deserialize, Serialize};

use crate::boardposition::BoardPosition;
use crate::direction::Direction;

/// Represents the logical position of an object on the game board.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect(Component, Default, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub visual_priority: f32,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            visual_priority: 0.0,
        }
    }
}

impl Position {
    pub fn new_i64(x: i64, y: i64, z: i64) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            z: z as f32,
            visual_priority: 0.0,
        }
    }

    pub fn with_visual_priority(&self, visual_priority: f32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z,
            visual_priority,
        }
    }

    pub fn is_finite(&self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.z.is_finite()
            && self.visual_priority.is_finite()
    }

    pub fn lerp(&self, target: &Position, alpha: f32) -> Position {
        Position {
            x: self.x + (target.x - self.x) * alpha,
            y: self.y + (target.y - self.y) * alpha,
            z: self.z + (target.z - self.z) * alpha,
            visual_priority: self.visual_priority,
        }
    }

    pub fn into_visual_priority(mut self, visual_priority: f32) -> Self {
        self.visual_priority = visual_priority;
        self
    }

    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    pub fn same_x(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < 0.0001
    }

    pub fn same_y(&self, other: &Self) -> bool {
        (self.y - other.y).abs() < 0.0001
    }

    pub fn same_z(&self, other: &Self) -> bool {
        (self.z - other.z).abs() < 0.0001
    }

    pub fn same_xy(&self, other: &Self) -> bool {
        self.same_x(other) && self.same_y(other)
    }

    pub fn distance(&self, other: &Self) -> f32 {
        self.distance2(other).sqrt()
    }

    pub fn distance_zf(&self, other: &Self, zf: f32) -> f32 {
        self.distance2_zf(other, zf).sqrt()
    }

    pub fn distance2(&self, other: &Self) -> f32 {
        self.distance2_zf(other, 6.0)
    }

    pub fn distance2_zf(&self, other: &Self, zf: f32) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = (self.z - other.z) * zf;
        dx * dx + dy * dy + dz * dz
    }

    /// Calculate squared distance with Z component multiplied by 10 if on different floors.
    /// This is used for game physics where vertical separation significantly reduces interaction.
    pub fn weighted_distance_squared(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;

        // Check if they're on different floors by comparing rounded Z values
        let self_floor = self.z.round();
        let other_floor = other.z.round();

        let dz = if self_floor != other_floor {
            // Multiply Z component by 10 when on different floors
            (self.z - other.z) * 10.0
        } else {
            self.z - other.z
        };

        dx * dx + dy * dy + dz * dz
    }

    /// Calculate weighted distance (non-squared).
    pub fn weighted_distance(&self, other: &Self) -> f32 {
        self.weighted_distance_squared(other).sqrt()
    }

    pub fn distance_taxicab(&self, other: &Self) -> f32 {
        self.distance_taxicab_zf(other, 6.0)
    }

    pub fn distance_taxicab_zf(&self, other: &Self, zf: f32) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = (self.z - other.z) * zf;
        dx.abs() + dy.abs() + dz.abs()
    }

    pub fn to_board_position(self) -> BoardPosition {
        BoardPosition {
            x: self.x.round() as i64,
            y: self.y.round() as i64,
            z: self.z.round() as i64,
        }
    }

    pub fn to_board_position_size(self, map_size: (usize, usize, usize)) -> BoardPosition {
        BoardPosition {
            x: (self.x.round() as i64).clamp(0, map_size.0 as i64 - 1),
            y: (self.y.round() as i64).clamp(0, map_size.1 as i64 - 1),
            z: (self.z.round() as i64).clamp(0, map_size.2 as i64 - 1),
        }
    }

    pub fn rotate_by_dir(&self, dir: &Direction) -> Self {
        let dir = dir.normalized();

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
            visual_priority: self.visual_priority,
        }
    }

    pub fn unrotate_by_dir(&self, dir: &Direction) -> Self {
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

impl MapEntities for Position {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

impl std::ops::Add<Direction> for &Position {
    type Output = Position;

    fn add(self, rhs: Direction) -> Self::Output {
        Position {
            x: self.x + rhs.dx,
            y: self.y + rhs.dy,
            z: self.z + rhs.dz,
            visual_priority: self.visual_priority,
        }
    }
}

impl std::ops::Add<Direction> for Position {
    type Output = Position;

    fn add(self, rhs: Direction) -> Self::Output {
        Position {
            x: self.x + rhs.dx,
            y: self.y + rhs.dy,
            z: self.z + rhs.dz,
            visual_priority: self.visual_priority,
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
            visual_priority: self.visual_priority - rhs.visual_priority,
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
            visual_priority: self.visual_priority - rhs.visual_priority,
        }
    }
}
