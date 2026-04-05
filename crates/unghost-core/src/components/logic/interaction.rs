use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use unspatial_core::position::Position;

/// Component for objects that are temporarily locked by ghost interactions.
///
/// Uses an absolute-time unlock timestamp instead of a `Timer` so this component
/// is safe to place in the logic layer (Rule D: logic components must not contain Timers).
#[derive(Component, Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Locked {
    /// The `Time::elapsed_secs_f64()` value at which the lock expires.
    pub unlock_at_secs: f64,
}

impl Default for Locked {
    fn default() -> Self {
        Self {
            unlock_at_secs: 0.0,
        }
    }
}

impl Locked {
    /// Create a new lock that expires `duration_secs` from `current_secs`
    /// (pass `time.elapsed_secs_f64()` as `current_secs`).
    pub fn new(current_secs: f64, duration_secs: f32) -> Self {
        Self {
            unlock_at_secs: current_secs + duration_secs as f64,
        }
    }

    /// Returns `true` when the lock duration has elapsed.
    pub fn is_finished(&self, current_secs: f64) -> bool {
        current_secs >= self.unlock_at_secs
    }
}

/// Semantic kind of an authoritative ghost-driven object motion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum InteractionMotionKind {
    Throw,
    Nudge,
    HauntedMove,
}

/// Authoritative motion state for ghost-driven object movement.
///
/// Logic owns and replicates this state, advances canonical `Position`, and
/// presentation derives local-only animation from it.
#[derive(Component, Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct InteractionMotion {
    pub start_pos: Position,
    pub end_pos: Position,
    /// `Time::elapsed_secs_f64()` at the moment this motion was inserted.
    pub start_secs: f64,
    /// How long the motion runs, in seconds.
    pub duration_secs: f32,
    pub kind: InteractionMotionKind,
}

impl Default for InteractionMotion {
    fn default() -> Self {
        Self {
            start_pos: Position::default(),
            end_pos: Position::default(),
            start_secs: 0.0,
            duration_secs: 0.0,
            kind: InteractionMotionKind::Nudge,
        }
    }
}

impl InteractionMotion {
    pub fn new_throw(start: Position, end: Position, current_secs: f64, duration: f32) -> Self {
        Self {
            start_pos: start,
            end_pos: end,
            start_secs: current_secs,
            duration_secs: duration,
            kind: InteractionMotionKind::Throw,
        }
    }

    pub fn new_nudge(start: Position, current_secs: f64, duration: f32) -> Self {
        let nudge_offset = Position {
            x: start.x + 0.3,
            y: start.y,
            z: start.z,
            visual_priority: start.visual_priority,
        };
        Self {
            start_pos: start,
            end_pos: nudge_offset,
            start_secs: current_secs,
            duration_secs: duration,
            kind: InteractionMotionKind::Nudge,
        }
    }

    pub fn new_nudge_to(start: Position, dest: Position, current_secs: f64, duration: f32) -> Self {
        Self {
            start_pos: start,
            end_pos: dest,
            start_secs: current_secs,
            duration_secs: duration,
            kind: InteractionMotionKind::Nudge,
        }
    }

    pub fn new_haunted_move(
        start: Position,
        end: Position,
        current_secs: f64,
        duration: f32,
    ) -> Self {
        Self {
            start_pos: start,
            end_pos: end,
            start_secs: current_secs,
            duration_secs: duration,
            kind: InteractionMotionKind::HauntedMove,
        }
    }

    /// Fraction of the animation elapsed [0.0, 1.0].
    pub fn fraction(&self, current_secs: f64) -> f32 {
        let elapsed = (current_secs - self.start_secs) as f32;
        (elapsed / self.duration_secs).clamp(0.0, 1.0)
    }

    /// Returns `true` when the animation is complete.
    pub fn is_finished(&self, current_secs: f64) -> bool {
        current_secs >= self.start_secs + self.duration_secs as f64
    }

    /// Current interpolated position, given the current elapsed time.
    pub fn current_position(&self, current_secs: f64) -> Position {
        let progress = self.fraction(current_secs);
        match self.kind {
            InteractionMotionKind::HauntedMove => Position {
                x: self.start_pos.x + (self.end_pos.x - self.start_pos.x) * progress,
                y: self.start_pos.y + (self.end_pos.y - self.start_pos.y) * progress,
                z: self.start_pos.z + (self.end_pos.z - self.start_pos.z) * progress,
                visual_priority: self.start_pos.visual_priority
                    + (self.end_pos.visual_priority - self.start_pos.visual_priority) * progress,
            },
            InteractionMotionKind::Throw => {
                let arc_height = 0.5;
                let arc_progress = 4.0 * arc_height * progress * (1.0 - progress);
                Position {
                    x: self.start_pos.x + (self.end_pos.x - self.start_pos.x) * progress,
                    y: self.start_pos.y + (self.end_pos.y - self.start_pos.y) * progress,
                    z: self.start_pos.z
                        + (self.end_pos.z - self.start_pos.z) * progress
                        + arc_progress,
                    visual_priority: self.start_pos.visual_priority
                        + (self.end_pos.visual_priority - self.start_pos.visual_priority)
                            * progress,
                }
            }
            InteractionMotionKind::Nudge => {
                let smooth = (progress * std::f32::consts::PI / 2.0).sin();
                Position {
                    x: self.start_pos.x + (self.end_pos.x - self.start_pos.x) * smooth,
                    y: self.start_pos.y + (self.end_pos.y - self.start_pos.y) * smooth,
                    z: self.start_pos.z + (self.end_pos.z - self.start_pos.z) * smooth,
                    visual_priority: self.start_pos.visual_priority
                        + (self.end_pos.visual_priority - self.start_pos.visual_priority) * smooth,
                }
            }
        }
    }
}
