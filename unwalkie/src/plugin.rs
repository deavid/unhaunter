use bevy::prelude::*;
use uncore::resources::potential_id_timer::PotentialIDTimer;
use unwalkiecore::{WalkiePlay, WalkieTalkingEvent};

pub struct UnhaunterWalkiePlugin;

impl Plugin for UnhaunterWalkiePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WalkieTalkingEvent>();
        app.init_resource::<WalkiePlay>();
        app.init_resource::<PotentialIDTimer>();

        crate::walkie_play::app_setup(app);
        crate::triggers::app_setup(app);
        crate::walkie_stats::app_setup(app);
        crate::walkie_level_stats::setup_walkie_level_systems(app);
        crate::focus_ring_system::app_setup(app);
    }
}
