use bevy::prelude::*;

#[derive(Clone, Debug, Default, Message)]
pub struct BoardTopologyToRebuild {
    pub lighting: bool,
    pub collision: bool,
}
