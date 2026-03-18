use bevy::prelude::*;
use std::time::Duration;

/// Event that triggers ambient sound muting with configurable fade timings.
/// Used by game systems to temporarily reduce ambient sound volume during important events.
#[derive(Message, Debug, Clone)]
pub struct AmbientSoundMuteEvent {
    /// How quickly to fade out (default: 3500ms)
    pub fade_out_duration: Duration,
    /// How long to stay muted (default: 2500ms)
    pub mute_duration: Duration,
    /// How long to fade back in (default: 15s)
    pub fade_in_duration: Duration,
    /// Volume reduction factor during mute phase (default: 3.0 = reduce volume 3x)
    /// 1.0 = no reduction, 3.0 = 1/3 volume reduction, 10.0 = 1/10th volume
    pub reduction_factor: f32,
}

impl Default for AmbientSoundMuteEvent {
    fn default() -> Self {
        Self {
            fade_out_duration: Duration::from_millis(3500),
            mute_duration: Duration::from_millis(2500),
            fade_in_duration: Duration::from_secs(15),
            reduction_factor: 3.0,
        }
    }
}
