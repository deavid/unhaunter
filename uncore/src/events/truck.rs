use bevy::prelude::*;

#[derive(Clone, Debug, Event, PartialEq, Eq)]
pub enum TruckUIEvent {
    EndMission,
    ExitTruck,
    CraftRepellent,
}
