use crate::metrics;
use bevy::prelude::*;
use ungearitems_core::events::RequestCraftRepellent;

pub struct UnhaunterGearItemsCorePlugin;

impl Plugin for UnhaunterGearItemsCorePlugin {
    fn build(&self, app: &mut App) {
        crate::registration::register_all(app);
        crate::net_state::app_setup(app);
        app.add_message::<RequestCraftRepellent>();
    }
}

pub struct UnhaunterGearItemsPresentationPlugin;

impl Plugin for UnhaunterGearItemsPresentationPlugin {
    fn build(&self, app: &mut App) {
        crate::components::quartz::app_setup(app);
        crate::components::salt::app_setup(app);
        crate::components::sage::app_setup(app);
        crate::components::thermometer::app_setup(app);
        crate::components::emfmeter::app_setup(app);
        crate::components::recorder::app_setup(app);
        crate::components::flashlight::app_setup(app);
        crate::components::geigercounter::app_setup(app);
        crate::components::uvtorch::app_setup(app);
        crate::components::videocam::app_setup(app);
        crate::components::redtorch::app_setup(app);
        crate::components::photocam::app_setup(app);
        crate::components::spiritbox::app_setup(app);
        crate::components::ionmeter::app_setup(app);
        crate::components::thermalimager::app_setup(app);
        crate::components::estaticmeter::app_setup(app);
        crate::components::compass::app_setup(app);
        crate::components::motionsensor::app_setup(app);
        crate::components::repellentflask::app_setup(app);

        metrics::register_all(app);
    }
}
