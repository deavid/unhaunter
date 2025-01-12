use crate::components::{MenuEvBack, MenuEvent, SettingsState};
use crate::{menu_ui, systems};
use bevy::prelude::*;
use uncore::states::AppState;

pub struct UnhaunterMenuSettingsPlugin;

impl Plugin for UnhaunterMenuSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SettingsState>()
            .add_systems(OnEnter(AppState::SettingsMenu), menu_ui::setup_ui)
            .add_systems(OnExit(AppState::SettingsMenu), menu_ui::cleanup)
            .add_systems(
                Update,
                (
                    systems::handle_input,
                    systems::item_highlight_system,
                    systems::menu_routing_system,
                    systems::menu_back_event,
                )
                    .run_if(in_state(AppState::SettingsMenu)),
            )
            .add_event::<MenuEvent>()
            .add_event::<MenuEvBack>();
    }
}
