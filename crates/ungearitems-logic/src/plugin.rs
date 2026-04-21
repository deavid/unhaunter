use bevy::prelude::*;
use unreplicon_core::resources::AuthorityRole;

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            crate::systems::common::system_electronic_interference,
            crate::systems::common::system_battery_drain,
            crate::systems::common::update_repellentflask_skeleton,
            crate::systems::common::handle_craft_repellent_request
                .run_if(resource_exists::<AuthorityRole>),
            crate::systems::salt::update_salt_skeleton,
            crate::systems::salt::salt_pile_system.run_if(resource_exists::<AuthorityRole>),
            crate::systems::salt::progress_salt_pile_arming
                .run_if(resource_exists::<AuthorityRole>),
            crate::systems::salt::salt_pile_cleanup_system.run_if(resource_exists::<AuthorityRole>),
            crate::systems::sage::update_sage_skeleton,
            crate::systems::sage::sage_authority_system.run_if(resource_exists::<AuthorityRole>),
            crate::systems::equipment::update_uvtorch_skeleton,
            crate::systems::equipment::update_redtorch_skeleton,
            crate::systems::equipment::update_flashlight_skeleton,
            crate::systems::equipment::update_videocam_skeleton,
            crate::systems::equipment::update_quartz_skeleton,
        )
            .run_if(in_state(uncommon_states_core::UIContextState::InGame)),
    );
}

pub struct UnhaunterGearItemsLogicPlugin;

impl Plugin for UnhaunterGearItemsLogicPlugin {
    fn build(&self, app: &mut App) {
        crate::metrics::register_all(app);
        crate::net_state::app_setup(app);
        app_setup(app);
        app.add_message::<ungearitems_core::events::RequestCraftRepellent>();
        app.add_message::<ungearitems_core::events::RepellentUsedEvent>();
        app.add_message::<ungearitems_core::events::QuartzCrackedEvent>();
    }
}
