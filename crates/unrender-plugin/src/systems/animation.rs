use bevy::prelude::*;

use unrender_std::components::animation::AnimationTimer;
use unrender_std::materials::CustomMaterial1;
use untypes_core::states::GameState;

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationTimer,
        Option<&mut Sprite>,
        Option<&MeshMaterial2d<CustomMaterial1>>,
    )>,
    mut materials: ResMut<Assets<CustomMaterial1>>,
) {
    for (mut anim, mut sprite, mat) in query.iter_mut() {
        let delta = time.delta();
        if let Some(idx) = anim.tick(delta) {
            if let Some(sprite) = sprite.as_mut()
                && let Some(texture_atlas) = sprite.texture_atlas.as_mut()
            {
                texture_atlas.index = idx;
            }
            if let Some(mat) = mat
                && let Some(material) = materials.get_mut(mat)
            {
                material.data.sheet_idx = idx as u32;
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, animate_sprite.run_if(in_state(GameState::None)));
}
