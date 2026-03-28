use bevy::prelude::*;

/// Emitted by `unprofile-plugin` at mission start.
/// Carries the insurance deposit amount staked for the current mission.
/// Consumed by `uncareer-plugin` to cache the deposit for reward calculation.
#[derive(Debug, Clone, Message)]
pub struct DepositStakedEvent {
    pub amount: i64,
}
