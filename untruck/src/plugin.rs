use bevy::prelude::*;
use uncore::events::truck::TruckUIEvent;
use uncore::resources::ghost_guess::GhostGuess;
use uncore::states::{AppState, GameState};

use super::loadoutui::{self, EventButtonClicked};
use super::{evidence, journal, sanity, systems, ui};

pub struct UnhaunterTruckPlugin;

impl Plugin for UnhaunterTruckPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), ui::setup_ui)
            .add_systems(OnExit(AppState::InGame), systems::cleanup)
            .add_systems(OnEnter(GameState::Truck), systems::show_ui)
            .add_systems(OnExit(GameState::Truck), systems::hide_ui)
            .add_event::<TruckUIEvent>()
            .add_event::<EventButtonClicked>()
            .init_resource::<GhostGuess>()
            .add_systems(Update, systems::keyboard)
            .add_systems(Update, journal::ghost_guess_system)
            .add_systems(
                FixedUpdate,
                (journal::button_system, sanity::update_sanity).run_if(in_state(GameState::Truck)),
            )
            .add_systems(
                Update,
                (
                    systems::truckui_event_handle,
                    ui::update_tab_interactions,
                    loadoutui::update_loadout_buttons,
                    loadoutui::button_clicked,
                ),
            );
        evidence::app_setup(app);
    }
}
