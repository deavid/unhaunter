use bevy::prelude::*;
use uninteraction_core::events::RoomChangedEvent;

pub struct ClassicModeGameplayPlugin;

impl Plugin for ClassicModeGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RoomChangedEvent>();
        crate::boot::app_setup(app);
        crate::simulation::app_setup(app);
        crate::environmental_mechanics::app_setup(app);
        crate::roomchanged::app_setup(app);
        crate::object_charge::app_setup(app);
        crate::evidence_perception::app_setup(app);
    }
}
