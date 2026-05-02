use bevy::prelude::*;

use crate::resources::ProcManConfig;
use crate::resources::TransportConfig;
use crate::systems;

/// Statically declares whether this process is acting as a server or a client.
/// Set once at plugin construction time and never changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkRole {
    Server,
    Client,
}

pub struct UnrepliconTransportPlugin {
    pub transport_config: TransportConfig,
    pub procman_config: ProcManConfig,
    pub network_role: NetworkRole,
}

impl Plugin for UnrepliconTransportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.transport_config.clone());
        app.insert_resource(self.procman_config.clone());
        systems::app_setup(app, self.network_role);
    }
}
