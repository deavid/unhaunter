use bevy::prelude::*;
use bevy_replicon::prelude::{SendMode, ToClients};
use rand::RngExt;
use unaudiospatial_core::emitter::LocalAudioEmitter;
use uncommon_app_core::random_seed;
use unghost_core::events::GhostAudioMessage;
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

pub(crate) fn emit_ghost_audio(
    sound_file: String,
    volume: f32,
    position: Position,
    local_audio: &mut LocalAudioEmitter,
    ev_audio: &mut MessageWriter<ToClients<GhostAudioMessage>>,
    has_local_player: bool,
) {
    if has_local_player {
        local_audio.play_audio(sound_file.clone(), volume, &position);
    }

    ev_audio.write(ToClients {
        mode: SendMode::Broadcast,
        message: GhostAudioMessage {
            sound_file,
            volume,
            position,
        },
    });
}

/// Execute a roar decision
pub(crate) fn execute_roar_decision(
    roar_decision: &RoarDecision,
    last_roar: &mut f32,
    ghost_position: &Position,
    local_audio: &mut LocalAudioEmitter,
    ev_audio: &mut MessageWriter<ToClients<GhostAudioMessage>>,
    has_local_player: bool,
) {
    if roar_decision.should_play_now {
        let roar_time_threshold = roar_decision.time_override.unwrap_or(3.0);
        if *last_roar > roar_time_threshold
            && let Some(roar_sound) = roar_decision.roar_type.get_sound()
        {
            emit_ghost_audio(
                roar_sound,
                roar_decision.roar_type.get_volume(),
                *ghost_position,
                local_audio,
                ev_audio,
                has_local_player,
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
