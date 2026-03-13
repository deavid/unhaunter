use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Data structure for the Quartz Stone consumable.
#[derive(Component, Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
pub struct QuartzStoneData {
    /// Number of cracks in the stone (0-3).
    pub cracks: u8,
}

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct QuartzStoneSkin {
    /// Bonus time for recently cracked
    pub cracked_time: f32,
    /// Amount of energy absorbed from the ghost - what produces the cracks.
    pub energy_absorbed: f32,
}
