use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct FadeOut {
    pub timer: Timer,
    pub roared: bool,
}

impl FadeOut {
    pub(crate) fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            roared: false,
        }
    }
}
