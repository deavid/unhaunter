use bevy::prelude::*;
use uncommon_app_core::states::AppState;
use uninput_core::components::PlayerInput;
use uninput_core::resources::MissionInputFocus;
use uninput_core::states::InGameUiState;

/// Derives MissionInputFocus from AppState + InGameUiState.
/// A single canonical answer to "should the mission window receive input?":
/// - Must be in AppState::InGame (not in main menu, lobby, summary, etc.)
/// - AND InGameUiState must be Running (no modal: not Paused, not in Truck, not in NPC dialog)
fn derive_mission_input_focus(
    app_state: Res<State<AppState>>,
    game_state: Res<State<InGameUiState>>,
    mut focus: ResMut<MissionInputFocus>,
) {
    focus.has_focus =
        *app_state.get() == AppState::InGame && *game_state.get() == InGameUiState::Running;
}

/// Resets stale input the frame focus is lost, preventing drift/walking-in-place during modals.
/// When a modal opens, input readers stop writing to PlayerInput. But the last frame's
/// movement vector is still present. Executors like apply_movement_intent would keep
/// the player drifting. This system zeros PlayerInput entirely when focus is lost.
fn reset_stale_input_on_focus_loss(
    mut q_input: Query<&mut PlayerInput>,
    focus: Res<MissionInputFocus>,
    mut had_focus: Local<bool>,
) {
    let has_focus = focus.has_focus;
    if *had_focus && !has_focus {
        // Focus was just lost this frame. Zero all input to prevent stale drift.
        for mut input in &mut q_input {
            input.zero();
        }
    }
    *had_focus = has_focus;
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        PreUpdate,
        (derive_mission_input_focus, reset_stale_input_on_focus_loss).chain(),
    );
}
