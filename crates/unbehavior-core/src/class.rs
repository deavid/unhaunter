use crate::traits::AutoSerialize;
use bevy::reflect::Reflect;
use bevy::reflect::std_traits::ReflectDefault;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash, Reflect)]
#[reflect(Default)]
pub enum Class {
    Floor,
    Wall,
    LowWall,
    Door,
    Switch,
    RoomSwitch,
    Breaker,
    Doorway,
    Decor,
    Item,
    Furniture,
    PlayerSpawn,
    GhostSpawn,
    VanEntry,
    RoomDef,
    WallLamp,
    FloorLamp,
    TableLamp,
    WallDecor,
    CeilingLight,
    StreetLight,
    CandleLight,
    Appliance,
    Van,
    Window,
    InvisibleWall,
    CornerWall,
    StairsUp,
    StairsDown,
    NPC,
    FakeGhost,
    FakeBreach,
    #[default]
    None,
}

impl AutoSerialize for Class {}
