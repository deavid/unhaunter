use bevy::prelude::*;
use unghost_core::resources::potential_id_timer::PotentialIDTimer;
use unwalkie_core::events::walkie_types::WalkieTalkingEvent;
use unwalkie_core::resources::WalkiePlay;

use crate::metrics;

pub struct UnhaunterWalkiePlugin;

impl Plugin for UnhaunterWalkiePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<WalkieTalkingEvent>();
        app.init_resource::<WalkiePlay>();
        app.init_resource::<PotentialIDTimer>();

        crate::walkie_play::app_setup(app);
        crate::triggers::setup::app_setup(app);
        crate::walkie_stats::app_setup(app);
        crate::walkie_level_stats::setup_walkie_level_systems(app);
        crate::focus_ring_system::app_setup(app);

        metrics::register_all(app);
    }
}
