use bevy::prelude::*;
use unspatial_core::position::Position;

/// Component for objects that are temporarily locked by ghost interactions
#[derive(Component)]
pub(crate) struct Locked(pub Timer);

/// Component for animating object movement during ghost interactions
#[derive(Component)]
pub(crate) struct Tween {
    pub start_pos: Position,
    pub end_pos: Position,
    pub timer: Timer,
    pub ease_fn: TweenEase,
}

/// Different easing functions for object animations
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TweenEase {
    /// Linear interpolation - constant speed
    Linear,
    /// Parabolic arc for thrown objects
    ParabolicArc,
    /// Smooth sine ease out for nudges
    SineEaseOut,
}

impl Tween {
    /// Create a new tween for throwing objects with parabolic arc
    pub(crate) fn new_throw(start: Position, end: Position, duration: f32) -> Self {
        Self {
            start_pos: start,
            end_pos: end,
            timer: Timer::from_seconds(duration, TimerMode::Once),
            ease_fn: TweenEase::ParabolicArc,
        }
    }

    /// Create a new tween for nudging objects with quick back-and-forth
    pub(crate) fn new_nudge(start: Position, duration: f32) -> Self {
        // For nudge, we move slightly forward then back to original position
        let nudge_offset = Position {
            x: start.x + 0.3, // Small forward movement
            y: start.y,
            z: start.z,
            visual_priority: start.visual_priority,
        };
        Self {
            start_pos: start,
            end_pos: nudge_offset,
            timer: Timer::from_seconds(duration, TimerMode::Once),
            ease_fn: TweenEase::SineEaseOut,
        }
    }

    /// Create a new tween for nudging objects towards a specific destination (typically floor)
    pub(crate) fn new_nudge_to(start: Position, dest: Position, duration: f32) -> Self {
        Self {
            start_pos: start,
            end_pos: dest,
            timer: Timer::from_seconds(duration, TimerMode::Once),
            ease_fn: TweenEase::SineEaseOut,
        }
    }

    /// Create a new tween for haunted movement with slow slide
    pub(crate) fn new_haunted_move(start: Position, end: Position, duration: f32) -> Self {
        Self {
            start_pos: start,
            end_pos: end,
            timer: Timer::from_seconds(duration, TimerMode::Once),
            ease_fn: TweenEase::Linear,
        }
    }

    /// Calculate the current interpolated position based on timer progress
    pub(crate) fn current_position(&self) -> Position {
        let progress = self.timer.fraction();

        match self.ease_fn {
            TweenEase::Linear => Position {
                x: self.start_pos.x + (self.end_pos.x - self.start_pos.x) * progress,
                y: self.start_pos.y + (self.end_pos.y - self.start_pos.y) * progress,
                z: self.start_pos.z + (self.end_pos.z - self.start_pos.z) * progress,
                visual_priority: self.start_pos.visual_priority
                    + (self.end_pos.visual_priority - self.start_pos.visual_priority) * progress,
            },
            TweenEase::ParabolicArc => {
                // Create a parabolic arc for thrown objects
                let linear_progress = progress;
                let arc_height = 0.5; // Peak height of the arc
                let arc_progress = 4.0 * arc_height * linear_progress * (1.0 - linear_progress);

                Position {
                    x: self.start_pos.x + (self.end_pos.x - self.start_pos.x) * linear_progress,
                    y: self.start_pos.y + (self.end_pos.y - self.start_pos.y) * linear_progress,
                    z: self.start_pos.z
                        + (self.end_pos.z - self.start_pos.z) * linear_progress
                        + arc_progress,
                    visual_priority: self.start_pos.visual_priority
                        + (self.end_pos.visual_priority - self.start_pos.visual_priority)
                            * linear_progress,
                }
            }
            TweenEase::SineEaseOut => {
                // Smooth ease out using sine
                let smooth_progress = (progress * std::f32::consts::PI / 2.0).sin();

                Position {
                    x: self.start_pos.x + (self.end_pos.x - self.start_pos.x) * smooth_progress,
                    y: self.start_pos.y + (self.end_pos.y - self.start_pos.y) * smooth_progress,
                    z: self.start_pos.z + (self.end_pos.z - self.start_pos.z) * smooth_progress,
                    visual_priority: self.start_pos.visual_priority
                        + (self.end_pos.visual_priority - self.start_pos.visual_priority)
                            * smooth_progress,
                }
            }
        }
    }
}

/// Component for visual effects particles spawned during object interactions
#[derive(Component, Debug)]
pub(crate) struct InteractionParticle {
    pub life: f32,
    pub max_life: f32,
    pub velocity: Vec3,
    pub particle_type: InteractionParticleType,
}

/// Different types of visual particles for ghost interactions
#[derive(Debug, Clone, Copy)]
pub(crate) enum InteractionParticleType {
    /// Dust particles when objects move/land
    Dust,
    /// Trail particles for thrown objects
    Trail,
    /// Creepy glow particles for haunted movement
    HauntedGlow,
    /// Sparks for electrical interactions (breaker trips)
    Spark,
}

/// Component for objects that should have motion blur during fast movement
#[derive(Component, Debug)]
pub(crate) struct MotionBlur {
    pub intensity: f32,
    pub previous_position: Position,
}

/// Component for door lock visual indicator
#[derive(Component, Debug)]
pub(crate) struct LockIndicator {
    pub pulse_timer: Timer,
    pub base_alpha: f32,
}

impl Default for LockIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl LockIndicator {
    pub(crate) fn new() -> Self {
        Self {
            pulse_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            base_alpha: 0.8,
        }
    }
}
