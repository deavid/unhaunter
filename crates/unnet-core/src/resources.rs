use crate::network_id::NetworkId;
use bevy::prelude::*;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalPlayer(pub Option<NetworkId>);

#[derive(Resource, Default, Debug, Clone)]
pub struct ChangedTiles(pub Vec<crate::messages::MapTileState>);
