use bevy::prelude::*;

#[derive(Clone, Debug, Message, PartialEq, Eq)]
pub enum TruckUIEvent {
    EndMission,
    ExitTruck,
    CraftRepellent,
}