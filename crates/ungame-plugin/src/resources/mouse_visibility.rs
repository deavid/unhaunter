use bevy::prelude::*;

/// Resource to track mouse cursor visibility state
#[derive(Resource, Default)]
pub struct MouseVisibility {
    pub is_visible: bool,
}
