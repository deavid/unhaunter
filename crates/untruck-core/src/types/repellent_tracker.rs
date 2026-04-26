use bevy::prelude::*;

/// Tracks the number of repellent bottles crafted and returned during the current mission.
/// This resource is used to enforce the per-mission craft limit based on difficulty.
#[derive(Resource, Component, Default, Debug, Clone, Reflect, PartialEq, serde::Serialize, serde::Deserialize)]
#[reflect(Component, Resource, Default, PartialEq)]
pub struct RepellentCraftTracker {
    pub crafted_count: u32,
    pub max_crafts: u32,
}

impl RepellentCraftTracker {
    pub fn remaining_crafts(&self) -> u32 {
        self.max_crafts.saturating_sub(self.crafted_count)
    }

    pub fn can_craft(&self) -> bool {
        self.crafted_count < self.max_crafts
    }

    pub fn craft(&mut self) {
        if self.can_craft() {
            self.crafted_count += 1;
        }
    }

    pub fn reset(&mut self, max_crafts: u32) {
        self.crafted_count = 0;
        self.max_crafts = max_crafts;
    }
}
