use bevy::prelude::*;

use uninvestigation_core::ghost::GhostType;

/// Emitted by `untruck-plugin` when the authority player holds the craft repellent button.
/// Handled by `ungearitems-plugin` on the authority node.
#[derive(Clone, Debug, Message)]
pub struct RequestCraftRepellent {
    pub ghost_type: GhostType,
}
