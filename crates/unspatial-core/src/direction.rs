use bevy::prelude::*;
use bevy_math::Vec3;
use serde::{Deserialize, Serialize};

use crate::position::Position;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect, Default)]
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
    pub fn is_finite(&self) -> bool {
        self.dx.is_finite() && self.dy.is_finite() && self.dz.is_finite()
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3 {
            x: self.dx,
            y: self.dy,
            z: self.dz,
        }
    }

    pub fn add_to_position(&self, rhs: &Position) -> Position {
        Position {
            x: self.dx + rhs.x,
            y: self.dy + rhs.y,
            z: self.dz + rhs.z,
            visual_priority: rhs.visual_priority,
        }
    }

    pub fn distance(&self) -> f32 {
        (self.dx * self.dx + self.dy * self.dy + self.dz * self.dz).sqrt()
    }

    pub fn distance2(&self) -> f32 {
        self.dx * self.dx + self.dy * self.dy + self.dz * self.dz
    }
    pub fn with_max_dist(&self, max_dist: f32) -> Self {
        let dst = self.distance() + 0.000000001;
        if dst < max_dist {
            return *self;
        }
        Self {
            dx: self.dx * max_dist / dst,
            dy: self.dy * max_dist / dst,
            dz: self.dz * max_dist / dst,
        }
    }
    pub fn normalized(&self) -> Self {
        let dst = self.distance() + 0.000000001;
        Self {
            dx: self.dx / dst,
            dy: self.dy / dst,
            dz: self.dz / dst,
        }
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

impl std::ops::Mul<f32> for Direction {
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

impl From<Vec3> for Direction {
    fn from(v: Vec3) -> Self {
        Self {
            dx: v.x,
            dy: v.y,
            dz: v.z,
        }
    }
}

impl From<Vec2> for Direction {
    fn from(v: Vec2) -> Self {
        Self {
            dx: v.x,
            dy: v.y,
            dz: 0.0,
        }
    }
}
