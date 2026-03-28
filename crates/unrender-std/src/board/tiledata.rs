use bevy::prelude::*;

#[derive(Component, Clone)]
pub enum PreMesh {
    Mesh(Mesh2d),
    Image {
        sprite_anchor: Vec2,
        image_handle: Handle<Image>,
    },
}
