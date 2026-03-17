use bevy::prelude::*;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum HintAnimationPhase {
    #[default]
    Idle,
    AnimatingIn,
    Visible,
    AnimatingOut,
}

#[derive(Resource, Debug)]
pub(crate) struct HintUiState {
    pub current_text: String,
    pub phase: HintAnimationPhase,
    pub animation_timer: Timer,
    pub slide_in_duration: Duration,
    pub visible_duration: Duration,
    pub slide_out_duration: Duration,
}

impl Default for HintUiState {
    fn default() -> Self {
        Self {
            current_text: String::new(),
            phase: HintAnimationPhase::Idle,
            animation_timer: Timer::from_seconds(0.0, TimerMode::Once),
            slide_in_duration: Duration::from_millis(300),
            visible_duration: Duration::from_secs(16),
            slide_out_duration: Duration::from_millis(300),
        }
    }
}
