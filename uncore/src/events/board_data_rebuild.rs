use bevy::prelude::*;

#[derive(Clone, Debug, Default, Event)]
pub struct BoardDataToRebuild {
    pub lighting: bool,
    pub collision: bool,
}
