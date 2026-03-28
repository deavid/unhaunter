use bevy::prelude::*;

use crate::resources::TransportConfig;
use crate::systems;

pub struct UnrepliconTransportPlugin {
    pub transport_config: TransportConfig,
    pub procman_config: crate::resources::ProcManConfig,
}

impl Plugin for UnrepliconTransportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.transport_config.clone());
        app.insert_resource(self.procman_config.clone());
        systems::app_setup(app);
    }
}
