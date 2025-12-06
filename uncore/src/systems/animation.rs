use bevy::prelude::*;

use crate::components::animation::AnimationTimer;

fn animate_sprite(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut Sprite)>) {
    for (mut anim, mut sprite) in query.iter_mut() {
        if let Some(idx) = anim.tick(time.delta())
            && let Some(texture_atlas) = sprite.texture_atlas.as_mut()
        {
            texture_atlas.index = idx;
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        animate_sprite.run_if(in_state(crate::states::GameState::None)),
    );
}
