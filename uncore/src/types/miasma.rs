use bevy::{math::Vec2};
use ndarray::Array3;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct MiasmaGrid {
    pub pressure_field: Array3<f32>,
    pub velocity_field: Array3<Vec2>,
    pub room_modifiers: HashMap<String, f32>, // Room ID -> Modifier
}
