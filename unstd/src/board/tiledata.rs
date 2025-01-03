use crate::materials::CustomMaterial1;
use bevy::prelude::*;
use uncore::behavior::Behavior;

#[derive(Component, Clone)]
pub enum PreMesh {
    Mesh(Mesh2d),
    Image {
        sprite_anchor: Vec2,
        image_handle: Handle<Image>,
    },
}

#[derive(Bundle, Clone)]
pub struct TileSpriteBundle {
    pub mesh: PreMesh,
    pub material: MeshMaterial2d<CustomMaterial1>,
    pub transform: Transform,
    pub visibility: Visibility,
}

#[derive(Clone)]
pub struct MapTileComponents {
    pub bundle: TileSpriteBundle,
    pub behavior: Behavior,
}
