use bevy::prelude::*;
use unwalkiecore::{WalkiePlay, WalkieTalkingEvent};

pub struct UnhaunterWalkiePlugin;

impl Plugin for UnhaunterWalkiePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WalkieTalkingEvent>()
            .init_resource::<WalkiePlay>();

        crate::walkie_play::app_setup(app);
        crate::triggers::app_setup(app);
        crate::walkie_stats::app_setup(app);
        crate::walkie_level_stats::setup_walkie_level_systems(app);
    }
}
