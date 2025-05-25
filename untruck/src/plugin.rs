use bevy::prelude::*;
use uncore::events::truck::TruckUIEvent;
use uncore::resources::ghost_guess::GhostGuess;
use uncore::states::{AppState, GameState};

use super::loadoutui::{self, EventButtonClicked};
use super::{journal, sanity, ui};

pub struct UnhaunterTruckPlugin;

impl Plugin for UnhaunterTruckPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), ui::setup_ui)
            .add_event::<TruckUIEvent>()
            .add_event::<EventButtonClicked>()
            .init_resource::<GhostGuess>()
            .add_systems(Update, journal::ghost_guess_system)
            .add_systems(
                FixedUpdate,
                (journal::button_system, sanity::update_sanity).run_if(in_state(GameState::Truck)),
            )
            .add_systems(
                Update,
                (
                    ui::update_tab_interactions,
                    loadoutui::update_loadout_buttons,
                    loadoutui::button_clicked,
                )
                    .run_if(in_state(GameState::Truck)),
            );

        super::evidence::app_setup(app);
        super::systems::app_setup(app);
    }
}
