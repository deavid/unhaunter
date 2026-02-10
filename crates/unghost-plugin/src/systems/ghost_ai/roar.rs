use bevy::prelude::*;
use rand::RngExt;
use unfoundation_core::random_seed;
use unsound_core::emitter::SoundEmitter;
use unspatial_core::position::Position;

/// Enables/disables debug logs for hunting behavior.
const DEBUG_HUNTS: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RoarType {
    Full,
    Dim,
    Snore,
    None,
}

impl RoarType {
    pub(crate) fn get_sound(&self) -> Option<String> {
        let roar_sounds = match self {
            RoarType::Full => vec![
                "sounds/ghost-roar-1.ogg",
                "sounds/ghost-roar-2.ogg",
                "sounds/ghost-roar-3.ogg",
                "sounds/ghost-roar-4.ogg",
            ],
            RoarType::Dim => vec![
                "sounds/ghost-effect-1.ogg",
                "sounds/ghost-effect-2.ogg",
                "sounds/ghost-effect-3.ogg",
                "sounds/ghost-effect-4.ogg",
            ],
            RoarType::Snore => vec![
                "sounds/ghost-snore-1.ogg",
                "sounds/ghost-snore-2.ogg",
                "sounds/ghost-snore-3.ogg",
                "sounds/ghost-snore-4.ogg",
            ],
            RoarType::None => return None,
        };

        roar_sounds
            .get(random_seed::rng().random_range(0..roar_sounds.len()))
            .map(|s: &&str| s.to_string())
    }

    pub(crate) fn get_volume(&self) -> f32 {
        match self {
            RoarType::Full => 1.0,
            RoarType::Dim => 0.9,
            RoarType::Snore => 0.3,
            RoarType::None => 0.0,
        }
    }
}

/// Decision about roar behavior with debugging information
#[derive(Debug)]
pub(crate) struct RoarDecision {
    pub roar_type: RoarType,
    pub should_play_now: bool,
    pub reason: RoarReason,
    pub time_override: Option<f32>,
}

/// Reason for roar decision (for debugging)
#[derive(Debug, Clone, Copy)]
pub(crate) enum RoarReason {
    HuntWarningStart,
    HuntingInProgress,
    RageBuildup,
    PeriodicSnore,
    None,
}

/// Execute a roar decision
pub(crate) fn execute_roar_decision(
    roar_decision: &RoarDecision,
    last_roar: &mut f32,
    ga: &mut SoundEmitter,
    ghost_position: &Position,
) {
    if roar_decision.should_play_now {
        let roar_time_threshold = roar_decision.time_override.unwrap_or(3.0);
        if *last_roar > roar_time_threshold
            && let Some(roar_sound) = roar_decision.roar_type.get_sound()
        {
            ga.play_audio(
                roar_sound,
                roar_decision.roar_type.get_volume(),
                ghost_position,
            );
            *last_roar = 0.0;

            if DEBUG_HUNTS {
                debug!(
                    "Ghost roar: {:?} - Reason: {:?}",
                    roar_decision.roar_type, roar_decision.reason
                );
            }
        }
    }
}
