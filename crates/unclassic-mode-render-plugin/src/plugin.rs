use bevy::prelude::*;

pub struct ClassicModeRenderPlugin;

impl Plugin for ClassicModeRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            crate::hydration::sync_ghost_visuals
                .run_if(in_state(untypes_core::states::AppState::InGame)),
        );

        crate::hydration::app_setup(app);
        crate::camera::app_setup(app);
    }
}
