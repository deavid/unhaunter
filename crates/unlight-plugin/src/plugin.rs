use bevy::prelude::*;
use unboard_core::BoardUpdateSet;
use untypes_core::states::AppState;

use unlight_core::resources::light_grid::LightGrid;

use crate::{audio, lighting_sim, maplight, metrics, systems::power_visuals};

pub struct UnhaunterLightCorePlugin;

impl Plugin for UnhaunterLightCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightGrid>()
            .add_systems(PreUpdate, lighting_sim::systems::init_light_grid)
            .add_systems(
                Update,
                lighting_sim::systems::prebake_lighting_on_level_ready,
            )
            .add_systems(
                PostUpdate,
                (
                    lighting_sim::systems::rebuild_lighting_field
                        .in_set(BoardUpdateSet::Lighting)
                        .after(BoardUpdateSet::Collision),
                    maplight::systems::gathering::player_visibility_system
                        .after(BoardUpdateSet::Lighting)
                        .run_if(in_state(AppState::InGame)),
                ),
            );
        metrics::register_all(app);
    }
}

pub struct UnhaunterLightPlugin;

impl Plugin for UnhaunterLightPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<unevents_core::events::ambient_sound_mute::AmbientSoundMuteEvent>();
        app.add_systems(
            PostUpdate,
            (
                power_visuals::update_power_visuals,
                maplight::systems::gathering::gather_flashlights_system,
                maplight::systems::gathering::update_exposure_system,
                maplight::systems::tiles::apply_lighting_to_tiles_system,
                maplight::systems::sprites::apply_lighting_to_sprites_system,
                maplight::systems::sprites::highlight_placement_tiles_system,
            )
                .chain()
                .after(BoardUpdateSet::Lighting)
                .run_if(in_state(AppState::InGame)),
        );
        audio::app_setup(app);
        maplight::systems::app_setup(app);
    }
}
