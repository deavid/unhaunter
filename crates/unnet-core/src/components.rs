use crate::messages::GearDetails;
use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct PendingGearState {
    pub target_on: Option<bool>,
    pub target_details: Option<GearDetails>,
    pub sent_at_secs: f64,
}

impl PendingGearState {
    pub fn new(time: f64) -> Self {
        Self {
            target_on: None,
            target_details: None,
            sent_at_secs: time,
        }
    }
}
