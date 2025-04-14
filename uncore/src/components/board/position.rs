use crate::random_seed;

use super::{
    EPSILON, PERSPECTIVE_X, PERSPECTIVE_Y, PERSPECTIVE_Z, boardposition::BoardPosition,
    direction::Direction,
};

use bevy::prelude::*;
use rand::Rng;

/// Represents the logical position of an object on the game board.
///
/// This component stores the object's 3D coordinates (`x`, `y`, `z`) in a logical
/// coordinate system, as well as a `global_z` value for fine-tuning the object's
/// vertical position in the isometric view.
///
/// The `to_screen_coord` method converts the logical position to screen
/// coordinates, applying the isometric perspective transformation. This
/// transformation is necessary to display the 3D game world in a 2D isometric view.
///
/// Other systems, such as the `apply_perspective` system, use the `Position`
/// component to update the `Transform` component of the object's sprite, ensuring
/// that the sprite is rendered at the correct position in the isometric view.
#[derive(Component, Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub global_z: f32,
}

impl Position {
    pub fn new_i64(x: i64, y: i64, z: i64) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            z: z as f32,
            global_z: 0 as f32,
        }
    }

    pub fn with_global_z(&self, global_z: f32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z,
            global_z,
        }
    }

    pub fn with_random(&self, range: f32) -> Self {
        let mut rng = random_seed::rng();
        Self {
            x: self.x + rng.random_range(-range..range),
            y: self.y + rng.random_range(-range..range),
            z: self.z,
            global_z: self.global_z,
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

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.same_x(other) && self.same_y(other) && self.same_z(other)
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
