pub(crate) mod power_visuals;
pub(crate) mod sprites;
pub(crate) mod tiles;

use bevy::prelude::*;
use unlight_core::sets::LightUpdateSet;

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (
            power_visuals::update_power_visuals,
            tiles::apply_lighting_to_tiles_system,
            sprites::apply_lighting_to_sprites_system,
            sprites::highlight_placement_tiles_system,
        )
            .chain()
            .in_set(LightUpdateSet::Apply),
    );
}
