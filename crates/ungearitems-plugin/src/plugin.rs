use crate::metrics;
use bevy::prelude::*;
use ungearitems_core::events::RequestCraftRepellent;
use untypes_core::roles::{AuthorityRole, LocalPlayerRole};

pub struct UnhaunterGearItemsCorePlugin;

impl Plugin for UnhaunterGearItemsCorePlugin {
    fn build(&self, app: &mut App) {
        crate::registration::register_all(app);
        app.add_message::<RequestCraftRepellent>();
    }
}

pub struct UnhaunterGearItemsPlugin;

impl Plugin for UnhaunterGearItemsPlugin {
    fn build(&self, app: &mut App) {
        crate::net_state::app_setup(app);
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

        app.add_systems(
            Update,
            crate::systems::handle_craft_repellent_request
                .run_if(resource_exists::<AuthorityRole>)
                .run_if(in_state(untypes_core::states::AppState::InGame)),
        );

        app.add_systems(
            Update,
            (
                crate::systems::system_electronic_interference,
                crate::systems::system_battery_drain,
            )
                .run_if(in_state(untypes_core::states::AppState::InGame))
                .run_if(resource_exists::<LocalPlayerRole>),
        );

        metrics::register_all(app);
    }
}
