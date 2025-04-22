use super::{Behavior, Orientation};
use crate::components::board::boardposition::BoardPosition;
use crate::types::tiledmap::map::MapLayer;
use bevy::{ecs::component::Component, log::warn, math::Vec3};

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Ground;
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Collision;
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Opaque;
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct UVSurface;
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Light;
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Door;
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Stairs {
    pub z: i32,
}
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct FloorItemCollidable;

#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub struct RoomState {
    pub room_delta: BoardPosition,
}

impl RoomState {
    pub fn new_for_room(orientation: &Orientation) -> Self {
        Self {
            room_delta: match orientation {
                Orientation::XAxis => BoardPosition { x: -1, y: 1, z: 0 },
                Orientation::YAxis => BoardPosition { x: -1, y: 1, z: 0 },
                Orientation::Both => BoardPosition::default(),
                Orientation::None => BoardPosition::default(),
            },
        }
    }

    pub fn with_opposite_side(orientation: &Orientation, opposite_side: bool) -> Self {
        if !opposite_side {
            return Self::new_for_room(orientation);
        }

        // If opposite_side is true, we'll use the opposite direction for the room delta
        Self {
            room_delta: match orientation {
                Orientation::XAxis => BoardPosition { x: -1, y: -1, z: 0 },
                Orientation::YAxis => BoardPosition { x: -1, y: -1, z: 0 },
                Orientation::Both => BoardPosition::default(),
                Orientation::None => BoardPosition::default(),
            },
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Interactive {
    pub on_activate_sound_file: String,
    pub on_deactivate_sound_file: String,
}

impl Interactive {
    pub fn new(activate: &str, deactivate: &str) -> Self {
        let on_activate_sound_file = activate.to_string();
        let on_deactivate_sound_file = deactivate.to_string();
        Self {
            on_activate_sound_file,
            on_deactivate_sound_file,
        }
    }

    pub fn sound_for_moving_into_state(&self, behavior: &Behavior) -> String {
        match behavior.cfg.state {
            super::TileState::On => self.on_activate_sound_file.clone(),
            super::TileState::Off => self.on_deactivate_sound_file.clone(),
            super::TileState::Open => self.on_activate_sound_file.clone(),
            super::TileState::Closed => self.on_deactivate_sound_file.clone(),
            super::TileState::Full => self.on_activate_sound_file.clone(),
            super::TileState::Partial => self.on_activate_sound_file.clone(),
            super::TileState::Minimum => self.on_activate_sound_file.clone(),
            super::TileState::None => self.on_deactivate_sound_file.clone(),
        }
    }

    pub fn control_point_delta(&self, behavior: &Behavior) -> Vec3 {
        match behavior.cfg.class {
            super::Class::Door => match behavior.cfg.orientation {
                super::Orientation::XAxis => Vec3::new(0.0, -0.25, 0.0),
                super::Orientation::YAxis => Vec3::new(0.25, 0.0, 0.0),
                _ => Vec3::ZERO,
            },
            _ => Vec3::ZERO,
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct NpcHelpDialog {
    pub dialog: String,
    pub seen: bool,
    pub trigger: f32,
}

impl NpcHelpDialog {
    pub fn new(classname: &str, variant: &str, layer: &MapLayer) -> Self {
        let key = format!("{classname}:{variant}:dialog");
        let dialog = match layer.user_properties.get(&key) {
            Some(p) => match p {
                tiled::PropertyValue::StringValue(v) => v.to_string(),
                _ => {
                    warn!(
                        "NPCHelpDialog was expecting a user property named {key:?} in the layer but it had an unsupported type - it must be text"
                    );
                    "".to_string()
                }
            },
            None => {
                warn!(
                    "NPCHelpDialog was expecting a user property named {key:?} in the layer but was not present"
                );
                "".to_string()
            }
        };
        Self {
            dialog,
            seen: false,
            trigger: 0.0,
        }
    }
}
