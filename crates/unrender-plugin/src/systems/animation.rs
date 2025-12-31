use bevy::prelude::*;

use unrender_std::components::animation::AnimationTimer;
use untypes_core::states::GameState;

fn animate_sprite(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut Sprite)>) {
    for (mut anim, mut sprite) in query.iter_mut() {
        let delta = time.delta();
        if let Some(idx) = anim.tick(delta)
            && let Some(texture_atlas) = sprite.texture_atlas.as_mut()
        {
            texture_atlas.index = idx;
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, animate_sprite.run_if(in_state(GameState::None)));
}
