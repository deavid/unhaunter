use serde::{Deserialize, Serialize};

/// Defines the behavioral personality of a ghost type, including interaction rates
/// for different actions at calm and angry states.
///
/// NOTE: This integrates with the existing ghost events system in unghost/src/ghost_events.rs
/// which already implements DoorSlam and LightFlicker behaviors. The new system will
/// expand these existing behaviors and add new interaction types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GhostPersonality {
    /// Rate for light/switch toggling actions per hour (calm, angry)
    /// This expands the existing LightFlicker behavior to include full toggles
    pub toggle_rate: (f32, f32),
    /// Rate for door slam actions per hour (calm, angry)
    /// This integrates with the existing DoorSlam behavior
    pub door_slam_rate: (f32, f32),
    /// Rate for door creak actions per hour (calm, angry)
    /// This is a new behavior - slow, subtle door movement
    pub door_creak_rate: (f32, f32),
    /// Rate for throwing objects per hour (calm, angry)
    /// This is a new behavior - fast object movement with parabolic arc
    pub throw_rate: (f32, f32),
    /// Rate for nudging objects per hour (calm, angry)
    /// This is a new behavior - small, quick object bump
    pub nudge_rate: (f32, f32),
    /// Rate for haunted object movement per hour (calm, angry)
    /// This is a new behavior - slow, eerie object sliding
    pub haunted_move_rate: (f32, f32),
    /// Rate for door locking per hour (calm, angry)
    /// This is a new behavior - temporary door locking with timer
    pub lock_rate: (f32, f32),
    /// Rate for breaker tripping per hour (calm, angry)
    /// This is a new behavior - cutting power to the entire house
    pub trip_breaker_rate: (f32, f32),
}

impl Default for GhostPersonality {
    fn default() -> Self {
        Self {
            toggle_rate: (15.0, 50.0),
            door_slam_rate: (8.0, 30.0),
            door_creak_rate: (20.0, 40.0),
            throw_rate: (5.0, 15.0),
            nudge_rate: (12.0, 35.0),
            haunted_move_rate: (8.0, 25.0),
            lock_rate: (2.0, 8.0),
            trip_breaker_rate: (1.0, 4.0),
        }
    }
}

impl GhostPersonality {
    /// Creates a calm ghost personality with very low interaction rates
    pub fn calm() -> Self {
        Self {
            toggle_rate: (8.0, 25.0),
            door_slam_rate: (2.0, 12.0),
            door_creak_rate: (12.0, 25.0),
            throw_rate: (3.0, 10.0),
            nudge_rate: (8.0, 18.0),
            haunted_move_rate: (5.0, 15.0),
            lock_rate: (1.0, 5.0),
            trip_breaker_rate: (0.5, 3.0),
        }
    }

    /// Creates an aggressive ghost personality with high interaction rates
    pub fn aggressive() -> Self {
        Self {
            toggle_rate: (25.0, 80.0),
            door_slam_rate: (15.0, 50.0),
            door_creak_rate: (20.0, 60.0),
            throw_rate: (12.0, 40.0),
            nudge_rate: (20.0, 65.0),
            haunted_move_rate: (15.0, 45.0),
            lock_rate: (5.0, 15.0),
            trip_breaker_rate: (3.0, 10.0),
        }
    }

    /// Creates a poltergeist personality focused on object manipulation
    pub fn poltergeist() -> Self {
        Self {
            toggle_rate: (30.0, 70.0),
            door_slam_rate: (20.0, 45.0),
            door_creak_rate: (15.0, 35.0),
            throw_rate: (25.0, 60.0),
            nudge_rate: (35.0, 75.0),
            haunted_move_rate: (25.0, 55.0),
            lock_rate: (8.0, 20.0),
            trip_breaker_rate: (5.0, 12.0),
        }
    }

    /// Creates a subtle ghost personality focused on atmospheric effects
    pub fn subtle() -> Self {
        Self {
            toggle_rate: (12.0, 35.0),
            door_slam_rate: (2.0, 8.0),
            door_creak_rate: (25.0, 50.0),
            throw_rate: (6.0, 18.0),
            nudge_rate: (10.0, 28.0),
            haunted_move_rate: (15.0, 35.0),
            lock_rate: (1.0, 5.0),
            trip_breaker_rate: (0.5, 3.0),
        }
    }
}
