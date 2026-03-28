use std::time::Duration;

use bevy::{
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};
use uninput_core::resources::MissionInputFocus;
use uninput_core::resources::MouseVisibility;
use unorchestrator_core::UIContextState;

fn system_hide_mouse(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut ev_cursor_moved: MessageReader<CursorMoved>,
    mut timer: Local<MouseTimer>,
    time: Res<Time>,
    focus: Res<MissionInputFocus>,
    mut mouse_visibility: ResMut<MouseVisibility>,
) {
    let cursor_moved = ev_cursor_moved.read().last();
    if cursor_moved.is_none() {
        timer.0.tick(time.delta());
    } else {
        timer.0.reset();
    }

    let visible = if focus.has_focus {
        !timer.0.is_finished()
    } else {
        true
    };
    mouse_visibility.is_visible = visible;

    // Query returns one window typically.
    for mut cursor_options in cursor_options_query.iter_mut() {
        cursor_options.visible = visible;
    }
}

pub(crate) struct MouseTimer(Timer);

impl Default for MouseTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(3), TimerMode::Once))
    }
}

/// System to ensure mouse cursor is visible when exiting the game state.
/// This prevents the cursor from staying permanently hidden after leaving the game.
fn show_mouse_cursor_on_exit(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut mouse_visibility: ResMut<MouseVisibility>,
) {
    mouse_visibility.is_visible = true;

    for mut cursor_options in cursor_options_query.iter_mut() {
        cursor_options.visible = true;
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<MouseVisibility>()
        .add_systems(Update, system_hide_mouse)
        .add_systems(OnExit(UIContextState::InGame), show_mouse_cursor_on_exit);
}
