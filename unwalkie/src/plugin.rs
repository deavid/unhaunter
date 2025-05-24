use bevy::prelude::*;
use uncore::resources::potential_id_timer::PotentialIDTimer;
use uncore::states::GameState;
use unwalkiecore::{WalkiePlay, WalkieTalkingEvent};

use crate::triggers::potential_id_prompt::potential_id_prompt_system;

use crate::triggers::{InTruckCraftPromptTimer, trigger_in_truck_craft_prompt_system};

pub struct UnhaunterWalkiePlugin;

impl Plugin for UnhaunterWalkiePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WalkieTalkingEvent>()
            .init_resource::<WalkiePlay>()
            .init_resource::<PotentialIDTimer>()
            .init_resource::<InTruckCraftPromptTimer>()
            .add_systems(
                Update,
                potential_id_prompt_system.run_if(in_state(GameState::None)),
            )
            .add_systems(
                Update,
                trigger_in_truck_craft_prompt_system.run_if(in_state(GameState::Truck)),
            );

        crate::walkie_play::app_setup(app);
        crate::triggers::app_setup(app);
        crate::walkie_stats::app_setup(app);
        crate::walkie_level_stats::setup_walkie_level_systems(app);
        crate::focus_ring_system::app_setup(app);

    }
}
