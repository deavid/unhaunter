use bevy::prelude::*;
use uninvestigation_core::resources::potential_id_timer::PotentialIDTimer;
use unwalkie_core::events::hint::OnScreenHintEvent;
use unwalkie_core::events::walkie_types::WalkieTalkingEvent;
use crate::metrics;

/// Client-only plugin that handles audio playback, hints, and metrics.
pub struct UnhaunterWalkieCorePlugin;

impl Plugin for UnhaunterWalkieCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<OnScreenHintEvent>();
        app.add_message::<WalkieTalkingEvent>();
        // WalkiePlay is initialized by unwalkie-logic's Plugin
        app.init_resource::<PotentialIDTimer>();
        metrics::register_all(app);
        crate::triggers::setup::app_setup(app);
    }
}

pub struct UnhaunterWalkiePlugin;

impl Plugin for UnhaunterWalkiePlugin {
    fn build(&self, app: &mut App) {
        crate::focus_ring_system::app_setup(app);
        crate::walkie_play::app_setup(app);
        crate::walkie_stats::app_setup(app);
        crate::walkie_level_stats::setup_walkie_level_systems(app);
    }
}
