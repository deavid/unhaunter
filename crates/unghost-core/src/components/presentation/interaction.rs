use crate::components::logic::interaction::{InteractionMotion, InteractionMotionKind};
use bevy::prelude::*;
use unspatial_core::position::Position;

/// Component for objects that should have motion blur during fast movement.
/// Local-only; never replicated. Owned by unghost-presentation.
#[derive(Component, Debug)]
pub struct MotionBlur {
    pub intensity: f32,
    pub previous_position: Position,
}

/// Local-only visual tween derived from authoritative `InteractionMotion`.
#[derive(Component, Debug, Clone, Copy)]
pub struct Tween {
    pub start_pos: Position,
    pub end_pos: Position,
    pub start_secs: f64,
    pub duration_secs: f32,
    pub ease_fn: TweenEase,
}

/// Different easing functions for local interaction visuals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TweenEase {
    Linear,
    ParabolicArc,
    SineEaseOut,
}

impl Tween {
    pub fn from_motion(motion: &InteractionMotion) -> Self {
        let ease_fn = match motion.kind {
            InteractionMotionKind::Throw => TweenEase::ParabolicArc,
            InteractionMotionKind::Nudge => TweenEase::SineEaseOut,
            InteractionMotionKind::HauntedMove => TweenEase::Linear,
        };

        Self {
            start_pos: motion.start_pos,
            end_pos: motion.end_pos,
            start_secs: motion.start_secs,
            duration_secs: motion.duration_secs,
            ease_fn,
        }
    }

    pub fn fraction(&self, current_secs: f64) -> f32 {
        let elapsed = (current_secs - self.start_secs) as f32;
        (elapsed / self.duration_secs).clamp(0.0, 1.0)
    }

    pub fn is_finished(&self, current_secs: f64) -> bool {
        current_secs >= self.start_secs + self.duration_secs as f64
    }
}

/// Component for visual effects particles spawned during object interactions.
/// Local-only; never replicated. Spawned and ticked by unghost-presentation.
#[derive(Component, Debug)]
pub struct InteractionParticle {
    pub life: f32,
    pub max_life: f32,
    pub velocity: Vec3,
    pub particle_type: InteractionParticleType,
}

/// Different types of visual particles for ghost interactions.
#[derive(Debug, Clone, Copy)]
pub enum InteractionParticleType {
    Dust,
    Trail,
    HauntedGlow,
    Spark,
}
