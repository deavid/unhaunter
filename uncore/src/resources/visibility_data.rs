use bevy::{prelude::*, utils::HashMap};

use crate::components::board::boardposition::BoardPosition;

#[derive(Clone, Debug, Resource, Default)]
pub struct VisibilityData {
    pub visibility_field: HashMap<BoardPosition, f32>,
}
