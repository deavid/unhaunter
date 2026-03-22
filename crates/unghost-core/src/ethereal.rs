use bevy::prelude::*;

/// TECHNICAL DEBT: This component violates Signal Direction (Tell/Don't Ask).
/// FIXME: unlight-plugin reads ghost-internal state from Ethereal (hunt_target, calm_time_secs, etc.).
/// This should be refactored to use event-driven signals from the ghost domain instead of
/// direct component queries. See: https://github.com/deavid/unhaunter/issues/XXX
///
/// Ghost ethereal/supernatural properties affecting visibility and rendering.
#[derive(Component, Debug, Clone, Copy)]
pub struct Ethereal {
    pub alpha_oscillation: f32,
    pub warp: f32,
    pub stability: f32,
    pub warning_active: bool,
    pub hunt_target: Option<Entity>,
    pub calm_time_secs: f32,
    pub hit_delta: f32,
    pub miss_delta: f32,
}
