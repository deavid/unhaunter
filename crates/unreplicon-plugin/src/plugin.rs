use bevy::prelude::*;
use bevy_replicon::server::visibility::registry::FilterRegistry;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;

use crate::systems;

pub struct UnrepliconPlugin {
    pub role_config: crate::systems::roles::RoleConfig,
    pub transport_config: unreplicon_transport::resources::TransportConfig,
    pub procman_config: unreplicon_transport::resources::ProcManConfig,
}

impl Plugin for UnrepliconPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.role_config.clone());
        app.insert_resource(self.transport_config.clone());
        app.insert_resource(self.procman_config.clone());
        systems::app_setup(app);
    }

    fn finish(&self, app: &mut App) {
        let bit =
            app.world_mut()
                .resource_scope(|world, mut filter_registry: Mut<FilterRegistry>| {
                    world.resource_scope(|world, mut registry: Mut<ReplicationRegistry>| {
                        filter_registry.register_scope::<Entity>(world, &mut registry)
                    })
                });
        app.insert_resource(GlobalFilterBit(bit));
    }
}

#[derive(Resource)]
pub struct GlobalFilterBit(pub bevy_replicon::server::visibility::filters_mask::FilterBit);
