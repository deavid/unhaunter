use std::time::Duration;

use bevy::{prelude::*, window::PrimaryWindow};

pub fn system_hide_mouse(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut ev_cursor_moved: EventReader<CursorMoved>,
    mut timer: Local<MouseTimer>,
    time: Res<Time>,
) {
    let cursor_moved = ev_cursor_moved.read().last();
    if cursor_moved.is_none() {
        timer.0.tick(time.delta());
    } else {
        timer.0.reset();
    }

    let visible = !timer.0.finished();
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
