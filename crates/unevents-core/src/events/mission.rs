use bevy::prelude::*;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionEvent {
    End,
}
