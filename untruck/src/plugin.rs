use bevy::prelude::*;
use uncore::events::truck::TruckUIEvent;
use uncore::resources::ghost_guess::GhostGuess;
use uncore::states::{AppState, GameState};

use super::loadoutui::{self, EventButtonClicked};
use super::{
    evidence, journal, sanity,
    systems::{
        journal_blinking_system::{
            update_journal_button_blinking_system, update_journal_ghost_blinking_system,
        },
        truck_ui_systems,
        ui_state_reset_system::reset_craft_button_highlight_on_truck_exit_system,
    },
    ui,
};

pub struct UnhaunterTruckPlugin;

impl Plugin for UnhaunterTruckPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), ui::setup_ui)
            .add_systems(OnExit(AppState::InGame), truck_ui_systems::cleanup)
            .add_systems(OnEnter(GameState::Truck), truck_ui_systems::show_ui)
            .add_systems(OnExit(GameState::Truck), truck_ui_systems::hide_ui)
            .add_event::<TruckUIEvent>()
            .add_event::<EventButtonClicked>()
            .init_resource::<GhostGuess>()
            .add_systems(Update, truck_ui_systems::keyboard)
            .add_systems(Update, journal::ghost_guess_system)
            .add_systems(Update, reset_craft_button_highlight_on_truck_exit_system)
            .add_systems(
                FixedUpdate,
                (
                    journal::button_system,
                    sanity::update_sanity,
                    // Run blinking systems after button_system to ensure border colors aren't overridden
                    update_journal_button_blinking_system.after(journal::button_system),
                    update_journal_ghost_blinking_system.after(journal::button_system),
                )
                    .run_if(in_state(GameState::Truck)),
            )
            .add_systems(
                Update,
                (
                    truck_ui_systems::hold_button_system,
                    truck_ui_systems::truckui_event_handle
                        .after(truck_ui_systems::hold_button_system),
                    ui::update_tab_interactions,
                    loadoutui::update_loadout_buttons,
                    loadoutui::button_clicked,
                )
                    .run_if(in_state(GameState::Truck)),
            );
        evidence::app_setup(app);
    }
}
