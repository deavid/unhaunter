use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub struct MapColor {
    pub color: Color,
}
