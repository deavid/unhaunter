use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unassets_core::resources::maps::Maps;
use undifficulty_core::plugin::UnhaunterDifficultyPlugin;
use unevents_core::events::hint::OnScreenHintEvent;
use unghost_core::resources::current_evidence_readings::CurrentEvidenceReadings;
use unmenu_core::mission_select::CurrentMissionSelectMode;
use unplayer_core::resources::PlayerInput;
use untypes_core::states::{AppState, GameState};
use unui_plugin::plugin::UnhaunterUiPlugin;

pub struct UnhaunterRootPlugin;

impl Plugin for UnhaunterRootPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UnhaunterDifficultyPlugin);
        app.add_plugins(UnhaunterUiPlugin);
        app.init_state::<AppState>()
            .init_state::<GameState>()
            .init_resource::<Maps>()
            .add_loading_state(
                LoadingState::new(AppState::Loading).continue_to_state(AppState::MainMenu),
            );

        app.init_resource::<CurrentEvidenceReadings>();
        app.init_resource::<CurrentMissionSelectMode>();
        app.init_resource::<unnoise_core::perlin::PerlinNoise>();
        app.init_resource::<unplayer_core::resources::game_config::GameConfig>();
        app.init_resource::<PlayerInput>();
        app.add_message::<OnScreenHintEvent>();

        arch_setup::app_setup(app);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod arch_setup {
    use bevy::prelude::*;

    fn set_fps_limiter(mut settings: ResMut<bevy_framepace::FramepaceSettings>) {
        settings.limiter = bevy_framepace::Limiter::from_framerate(60.0);
    }

    pub(crate) fn app_setup(app: &mut App) {
        app.add_plugins(bevy_framepace::FramepacePlugin)
            .add_systems(Startup, set_fps_limiter);
    }
}

#[cfg(target_arch = "wasm32")]
mod arch_setup {
    use bevy::prelude::*;

    pub fn app_setup(_app: &mut App) {}
}
