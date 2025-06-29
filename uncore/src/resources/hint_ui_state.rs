use bevy::prelude::*;
use std::time::Duration;

/// Represents the different animation phases for the on-screen hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HintAnimationPhase {
    #[default]
    Idle, // Not visible, or finished animating out
    AnimatingIn,
    Visible,
    AnimatingOut,
}

/// Resource to manage the state of the on-screen hint UI.
#[derive(Resource, Debug)]
pub struct HintUiState {
    /// The current text to display in the hint.
    pub current_text: String,
    /// The current animation phase of the hint.
    pub phase: HintAnimationPhase,
    /// Timer to manage the duration of animation phases and visibility.
    pub animation_timer: Timer,
    /// Configuration: Duration for the slide-in animation.
    pub slide_in_duration: Duration,
    /// Configuration: Duration the hint stays visible.
    pub visible_duration: Duration,
    /// Configuration: Duration for the slide-out animation.
    pub slide_out_duration: Duration,
}

impl Default for HintUiState {
    fn default() -> Self {
        Self {
            current_text: String::new(),
            phase: HintAnimationPhase::Idle,
            animation_timer: Timer::from_seconds(0.0, TimerMode::Once), // Duration will be set dynamically
            slide_in_duration: Duration::from_millis(300),
            visible_duration: Duration::from_secs(16),
            slide_out_duration: Duration::from_millis(300),
        }
    }
}
