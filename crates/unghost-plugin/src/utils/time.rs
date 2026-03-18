use bevy::{
    prelude::{Deref, DerefMut},
    time::{Timer, TimerMode},
};

#[derive(Deref, DerefMut)]
pub(crate) struct PrintingTimer(Timer);

impl Default for PrintingTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(60.0, TimerMode::Repeating))
    }
}
