use bevy::prelude::*;
use unwalkie_core::resources::WalkiePlay;

/// Loaded unconditionally on all peers (dedicated server, host, client).
/// Registers network message channels and server-side walkie systems.
pub struct UnhaunterWalkieLogicPlugin;

impl Plugin for UnhaunterWalkieLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WalkiePlay>();
        crate::walkie_net::app_setup_messages(app);
        crate::walkie_net::app_setup(app);
    }
}