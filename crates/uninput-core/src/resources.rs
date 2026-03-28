use bevy::prelude::*;

/// Resource to track mouse cursor visibility state.
#[derive(Resource, Default)]
pub struct MouseVisibility {
    pub is_visible: bool,
}

/// Resource to track whether the mission window has input focus.
/// When true, input readers should generate events and write to PlayerInput.
/// When false, input is gated (modal/menu has focus), and PlayerInput should be zeroed.
///
/// This is updated by `uninput-plugin` based on `InGameUiState`:
/// - `InGameUiState::Running` → `has_focus = true`
/// - All other states (Pause, Truck, NpcHelp) → `has_focus = false`
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissionInputFocus {
    pub has_focus: bool,
}
