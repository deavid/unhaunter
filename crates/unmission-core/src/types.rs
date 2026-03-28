use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionEvent {
    End,
}

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SimulationState {
    #[default]
    Unloaded, // No map loaded, all simulation systems should be idle
    Loading,  // Map geometry loaded, fields are being allocated
    Spawning, // All fields allocated, level content ready
    Ready,    // InGame + simulation is safe to run
    /// Mission has ended. Simulation ticks have stopped. Board entities are
    /// being despawned and arrays are being zeroed before returning to Unloaded.
    TearingDown,
}
