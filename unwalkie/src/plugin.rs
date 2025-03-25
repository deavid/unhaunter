use bevy::prelude::*;
use uncore::{events::walkie::WalkieEvent, resources::walkie::WalkiePlay};

pub struct UnhaunterWalkiePlugin;

impl Plugin for UnhaunterWalkiePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WalkieEvent>().init_resource::<WalkiePlay>();
        crate::walkie_play::app_setup(app);
    }
}
