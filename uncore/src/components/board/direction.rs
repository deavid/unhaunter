use super::{PERSPECTIVE_X, PERSPECTIVE_Y, PERSPECTIVE_Z, position::Position};

use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Direction {
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

impl Direction {
    pub fn new_right() -> Self {
        Self {
            dx: 1.0,
            dy: 0.0,
            dz: 0.0,
        }
    }

    pub fn zero() -> Self {
        Self {
            dx: 0.0,
            dy: 0.0,
            dz: 0.0,
        }
    }

    pub fn add_to_position(&self, rhs: &Position) -> Position {
        Position {
            x: self.dx + rhs.x,
            y: self.dy + rhs.y,
            z: self.dz + rhs.z,
            global_z: rhs.global_z,
        }
    }

    pub fn distance(&self) -> f32 {
        (self.dx * self.dx + self.dy * self.dy + self.dz * self.dz).sqrt()
    }

    pub fn distance2(&self) -> f32 {
        self.dx * self.dx + self.dy * self.dy + self.dz * self.dz
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
