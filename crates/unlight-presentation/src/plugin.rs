use bevy::prelude::*;
use unboard_core::BoardUpdateSet;
use uncommon_states_core::UIContextState;
use unlight_core::sets::LightUpdateSet;
use unmission_core::types::SimulationState;

use crate::{metrics, systems};

pub struct UnlightPresentationPlugin;

impl Plugin for UnlightPresentationPlugin {
    fn build(&self, app: &mut App) {
        metrics::register_all(app);
        app.configure_sets(
            PostUpdate,
            LightUpdateSet::Apply
                .after(BoardUpdateSet::Lighting)
                .after(LightUpdateSet::Gather)
                .run_if(in_state(UIContextState::InGame).and(in_state(SimulationState::Ready))),
        );
        systems::app_setup(app);
    }
}
