use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Defines the keyboard controls for a player.
#[derive(Resource, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlKeys {
    /// Key for moving up.
    pub up: KeyCode,
    /// Key for moving down.
    pub down: KeyCode,
    /// Key for moving left.
    pub left: KeyCode,
    /// Key for moving right.
    pub right: KeyCode,
    /// Key for interacting with objects (doors, switches, etc.).
    pub activate: KeyCode,
    /// Key for grabbing objects.
    pub grab: KeyCode,
    /// Key for dropping objects.
    pub drop: KeyCode,
    /// Key for triggering the left-hand item (e.g., flashlight).
    pub torch: KeyCode,
    /// Key for triggering the right-hand item (e.g., EMF reader).
    pub trigger: KeyCode,
    /// Key for cycling through inventory items.
    pub cycle: KeyCode,
    /// Key for swapping left and right hand items.
    pub swap: KeyCode,
    /// Key for changing the evidence selection in the quick menu.
    pub change_evidence: KeyCode,
    /// Key for running (hold to move faster).
    pub run: KeyCode,
    /// Key for temporarily looking on the left hand gear
    pub left_hand_look: KeyCode,
    /// Key for toggling looking on the left hand gear
    pub left_hand_toggle: KeyCode,
    /// Key for moving the camera up.
    pub camera_up: KeyCode,
    /// Key for moving the camera down.
    pub camera_down: KeyCode,
    /// Key for moving the camera left.
    pub camera_left: KeyCode,
    /// Key for moving the camera right.
    pub camera_right: KeyCode,
}

impl Default for ControlKeys {
    fn default() -> Self {
        Self {
            up: KeyCode::KeyW,
            down: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            activate: KeyCode::KeyE,
            trigger: KeyCode::KeyR,
            torch: KeyCode::Tab,
            cycle: KeyCode::KeyQ,
            swap: KeyCode::KeyT,
            drop: KeyCode::KeyG,
            grab: KeyCode::KeyF,
            change_evidence: KeyCode::KeyC,
            run: KeyCode::ShiftLeft,
            left_hand_look: KeyCode::ControlLeft,
            left_hand_toggle: KeyCode::CapsLock,
            camera_up: KeyCode::ArrowUp,
            camera_down: KeyCode::ArrowDown,
            camera_left: KeyCode::ArrowLeft,
            camera_right: KeyCode::ArrowRight,
        }
    }
}

/// System for handling player movement, interaction, and collision.
///
/// This system processes player input, updates the player's position and
/// direction, handles interactions with interactive objects, and manages
/// collisions with the environment.
impl ControlKeys {
    pub const WASD: Self = ControlKeys {
        up: KeyCode::KeyW,
        down: KeyCode::KeyS,
        left: KeyCode::KeyA,
        right: KeyCode::KeyD,
        activate: KeyCode::KeyE,
        trigger: KeyCode::KeyR,
        torch: KeyCode::Tab,
        cycle: KeyCode::KeyQ,
        swap: KeyCode::KeyT,
        drop: KeyCode::KeyG,
        grab: KeyCode::KeyF,
        change_evidence: KeyCode::KeyC,
        run: KeyCode::ShiftLeft,
        left_hand_look: KeyCode::ControlLeft,
        left_hand_toggle: KeyCode::CapsLock,
        camera_up: KeyCode::ArrowUp,
        camera_down: KeyCode::ArrowDown,
        camera_left: KeyCode::ArrowLeft,
        camera_right: KeyCode::ArrowRight,
    };
    pub const ARROWS: Self = ControlKeys {
        up: KeyCode::ArrowUp,
        down: KeyCode::ArrowDown,
        left: KeyCode::ArrowLeft,
        right: KeyCode::ArrowRight,
        activate: KeyCode::KeyE,
        trigger: KeyCode::KeyR,
        torch: KeyCode::Tab,
        cycle: KeyCode::KeyQ,
        swap: KeyCode::KeyT,
        drop: KeyCode::KeyG,
        grab: KeyCode::KeyF,
        change_evidence: KeyCode::KeyC,
        run: KeyCode::ShiftLeft,
        left_hand_look: KeyCode::ControlLeft,
        left_hand_toggle: KeyCode::CapsLock,
        camera_up: KeyCode::ArrowUp,
        camera_down: KeyCode::ArrowDown,
        camera_left: KeyCode::ArrowLeft,
        camera_right: KeyCode::ArrowRight,
    };
    pub const IJKL: Self = ControlKeys {
        up: KeyCode::KeyI,
        down: KeyCode::KeyK,
        left: KeyCode::KeyJ,
        right: KeyCode::KeyL,
        activate: KeyCode::KeyO,
        torch: KeyCode::KeyT,
        cycle: KeyCode::NonConvert,
        swap: KeyCode::NonConvert,
        grab: KeyCode::NonConvert,
        drop: KeyCode::NonConvert,
        trigger: KeyCode::NonConvert,
        change_evidence: KeyCode::NonConvert,
        run: KeyCode::ShiftRight,
        left_hand_look: KeyCode::ShiftRight,
        left_hand_toggle: KeyCode::Enter,
        camera_up: KeyCode::ArrowUp,
        camera_down: KeyCode::ArrowDown,
        camera_left: KeyCode::ArrowLeft,
        camera_right: KeyCode::ArrowRight,
    };
    pub const NONE: Self = ControlKeys {
        up: KeyCode::NonConvert,
        down: KeyCode::NonConvert,
        left: KeyCode::NonConvert,
        right: KeyCode::NonConvert,
        activate: KeyCode::NonConvert,
        torch: KeyCode::NonConvert,
        cycle: KeyCode::NonConvert,
        swap: KeyCode::NonConvert,
        grab: KeyCode::NonConvert,
        drop: KeyCode::NonConvert,
        trigger: KeyCode::NonConvert,
        change_evidence: KeyCode::NonConvert,
        run: KeyCode::NonConvert,
        left_hand_look: KeyCode::NonConvert,
        left_hand_toggle: KeyCode::NonConvert,
        camera_up: KeyCode::NonConvert,
        camera_down: KeyCode::NonConvert,
        camera_left: KeyCode::NonConvert,
        camera_right: KeyCode::NonConvert,
    };
}
