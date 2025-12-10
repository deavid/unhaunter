use bevy::prelude::*;

#[derive(Clone, Debug, Default, Message)]
pub struct BoardDataToRebuild {
    pub lighting: bool,
    pub collision: bool,
}
