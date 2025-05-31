use std::time::Duration;

use bevy::{prelude::*, window::PrimaryWindow};
use uncore::states::{AppState, GameState};

fn system_hide_mouse(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut ev_cursor_moved: EventReader<CursorMoved>,
    mut timer: Local<MouseTimer>,
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
) {
    let cursor_moved = ev_cursor_moved.read().last();
    if cursor_moved.is_none() {
        timer.0.tick(time.delta());
    } else {
        timer.0.reset();
    }

    let visible = if *app_state == AppState::InGame && *game_state == GameState::None {
        !timer.0.finished()
    } else {
        true
    };
    // Query returns one window typically.
    for mut window in windows.iter_mut() {
        window.cursor_options.visible = visible;
    }
}
pub struct MouseTimer(Timer);

impl Default for MouseTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(3), TimerMode::Once))
    }
}

/// System to ensure mouse cursor is visible when exiting the game state.
/// This prevents the cursor from staying permanently hidden after leaving the game.
fn show_mouse_cursor_on_exit(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    for mut window in windows.iter_mut() {
        window.cursor_options.visible = true;
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, system_hide_mouse)
        .add_systems(OnExit(AppState::InGame), show_mouse_cursor_on_exit);
}
