use bevy::prelude::*;
use unplayer_core::components::PlayerSprite;
use unrender_std::materials::CustomMaterial1;

pub(crate) fn player_style_system(
    mut query_players: Query<(&PlayerSprite, &mut MeshMaterial2d<CustomMaterial1>)>,
    mut materials: ResMut<Assets<CustomMaterial1>>,
) {
    for (sprite, mat_handle) in query_players.iter_mut() {
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let tint: LinearRgba = match sprite.id {
                1 => Color::srgba(0.8, 1.0, 0.8, 1.0).into(), // Host: Greenish
                2 => Color::srgba(1.0, 1.0, 0.8, 1.0).into(), // Client: Yellowish
                _ => Color::WHITE.into(),
            };
            if mat.data.color != tint {
                mat.data.color = tint;
            }
        }
    }
}
