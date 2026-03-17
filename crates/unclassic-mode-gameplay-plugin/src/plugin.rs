use bevy::prelude::*;

pub struct ClassicModeGameplayPlugin;

impl Plugin for ClassicModeGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<unevents_core::events::roomchanged::RoomChangedEvent>();
        crate::environmental_mechanics::app_setup(app);
        crate::roomchanged::app_setup(app);
        crate::object_charge::app_setup(app);
        crate::evidence_perception::app_setup(app);
    }
}
