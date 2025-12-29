use bevy::prelude::*;
use bevy_picking::PickingSystems;
use unpicking_core::*;
use crate::sprite_picking_backend::custom_sprite_picking;

/// Plugin that enables custom sprite picking for map sprites
#[derive(Clone)]
pub struct CustomSpritePickingPlugin;

impl Plugin for CustomSpritePickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CustomSpritePickingSettings>()
            .add_systems(
                PreUpdate,
                custom_sprite_picking.in_set(PickingSystems::Backend),
            );
    }
}
