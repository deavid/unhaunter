use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct HoverState {
    pub is_hovered: bool,
}
