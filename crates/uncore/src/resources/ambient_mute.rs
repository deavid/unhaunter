use bevy::prelude::*;
use std::time::Duration;

use crate::events::ambient_sound_mute::AmbientSoundMuteEvent;

/// Represents an active ambient sound mute effect with timing and fade logic.
/// Tracks elapsed time and calculates volume multipliers during fade-out, mute, and fade-in phases.
#[derive(Debug, Clone)]
pub struct ActiveMute {
    pub config: AmbientSoundMuteEvent,
    pub elapsed: Duration,
}

impl ActiveMute {
    /// Returns current volume multiplier (reduction_factor^-1 = reduced, 1.0 = normal)
    /// For example, with reduction_factor=10.0, this returns 0.1 during mute phase
    pub fn current_multiplier(&self) -> f32 {
        let total_duration = self.config.fade_out_duration
            + self.config.mute_duration
            + self.config.fade_in_duration;

        let mute_multiplier = 1.0 / self.config.reduction_factor; // e.g., 1/10 = 0.1 for 10x reduction

        if self.elapsed < self.config.fade_out_duration {
            // Fading out: 1.0 -> mute_multiplier
            let progress = self.elapsed.as_secs_f32() / self.config.fade_out_duration.as_secs_f32();
            1.0 - progress * (1.0 - mute_multiplier)
        } else if self.elapsed < self.config.fade_out_duration + self.config.mute_duration {
            // Muted: reduced volume (not silent)
            mute_multiplier
        } else if self.elapsed < total_duration {
            // Fading in: mute_multiplier -> 1.0 (with curve for perceived linearity)
            let fade_start = self.config.fade_out_duration + self.config.mute_duration;
            let fade_progress = (self.elapsed - fade_start).as_secs_f32()
                / self.config.fade_in_duration.as_secs_f32();

            // Apply curve for perceived linear volume change (sqrt for power->amplitude)
            let curved_progress = fade_progress.sqrt();
            mute_multiplier + curved_progress * (1.0 - mute_multiplier)
        } else {
            // Expired, remove this mute
            1.0
        }
    }

    /// Returns true if this mute effect has completed all phases and should be removed.
    pub fn is_expired(&self) -> bool {
        let total_duration = self.config.fade_out_duration
            + self.config.mute_duration
            + self.config.fade_in_duration;
        self.elapsed >= total_duration
    }
}

/// Controller that manages multiple active ambient sound mute effects.
/// Combines multiple mute effects multiplicatively to handle overlapping mutes.
#[derive(Resource, Debug, Default)]
pub struct AmbientMuteController {
    pub active_mutes: Vec<ActiveMute>,
}

impl AmbientMuteController {
    /// Returns the current mute multiplier as the product of all active mutes
    /// (reduction_factor^-1 = reduced volume, 1.0 = no muting effect)
    /// For example, one 10x reduction mute returns 0.1, two would return 0.01
    pub fn current_multiplier(&self) -> f32 {
        self.active_mutes
            .iter()
            .map(|mute| mute.current_multiplier())
            .product()
    }
}
