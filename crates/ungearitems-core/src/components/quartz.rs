use bevy::prelude::*;

/// Data structure for the Quartz Stone consumable.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct QuartzStoneData {
    /// Number of cracks in the stone (0-3).
    pub cracks: u8,
    /// Bonus time for recently cracked
    pub cracked_time: f32,
    /// Amount of energy absorbed from the ghost - what produces the cracks.
    pub energy_absorbed: f32,
}
