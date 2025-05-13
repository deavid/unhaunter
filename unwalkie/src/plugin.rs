use bevy::prelude::*;
use unwalkiecore::{WalkieEvent, WalkiePlay};

pub struct UnhaunterWalkiePlugin;

impl Plugin for UnhaunterWalkiePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WalkieEvent>().init_resource::<WalkiePlay>();

        crate::walkie_play::app_setup(app);
        crate::triggers::app_setup(app);
    }
}
