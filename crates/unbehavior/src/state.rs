use crate::traits::AutoSerialize;
use bevy::reflect::Reflect;
use bevy::reflect::std_traits::ReflectDefault;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash, Reflect)]
#[reflect(Default)]
pub enum TileState {
    // Switch states
    On,
    Off,
    // Door states
    Open,
    Closed,
    // Wall states
    Full,
    Partial,
    Minimum,
    // Default state
    #[default]
    None,
}

impl AutoSerialize for TileState {}
