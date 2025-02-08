use bevy::prelude::*;

use crate::components::animation::AnimationTimer;

pub fn animate_sprite(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut Sprite)>) {
    for (mut anim, mut sprite) in query.iter_mut() {
        if let Some(idx) = anim.tick(time.delta()) {
            if let Some(texture_atlas) = sprite.texture_atlas.as_mut() {
                texture_atlas.index = idx;
            }
        }
    }
}
