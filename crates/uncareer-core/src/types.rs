use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Career progression state (persists between missions)
#[derive(Debug, Clone, Resource, Component, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct CareerState {
    pub total_earnings: i64,
    pub total_xp: i64,
    pub player_level: u32,
}

/// Event emitted when career state is updated
#[derive(Debug, Clone, Event)]
pub struct CareerUpdatedEvent {
    pub earnings_delta: i64,
    pub xp_delta: i64,
}
