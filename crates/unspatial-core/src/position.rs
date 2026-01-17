use bevy::prelude::*;
use bevy_math::Vec3;

use crate::boardposition::BoardPosition;
use crate::direction::Direction;

// use unfoundation_core::random_seed; // TODO: move or handle

/// Represents the logical position of an object on the game board.
///
/// This component stores the object's 3D coordinates (`x`, `y`, `z`) in a logical
/// coordinate system, as well as a `visual_priority` value for fine-tuning the object's
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
    pub visual_priority: f32,
}

impl Position {
    pub fn new_i64(x: i64, y: i64, z: i64) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            z: z as f32,
            visual_priority: 0 as f32,
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

    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
            visual_priority: self.visual_priority,
        }
    }

    // pub fn with_random(&self, range: f32) -> Self {
    //     let mut rng = random_seed::rng();
    //     Self {
    //         x: self.x + rng.random_range(-range..range),
    //         y: self.y + rng.random_range(-range..range),
    //         z: self.z,
    //         visual_priority: self.visual_priority,
    //     }
    // }

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
        self.same_x(other) || self.same_y(other)
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
            visual_priority: self.visual_priority,
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
            visual_priority: self.visual_priority,
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
