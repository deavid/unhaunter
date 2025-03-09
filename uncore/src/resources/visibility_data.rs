use bevy::prelude::*;
use ndarray::Array3;

#[derive(Clone, Debug, Resource, Default)]
pub struct VisibilityData {
    pub visibility_field: Array3<f32>,
}
