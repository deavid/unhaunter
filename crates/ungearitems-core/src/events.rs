use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use uninvestigation_core::ghost::GhostType;

/// Emitted by `untruck-plugin` when the authority player holds the craft repellent button.
/// Handled by `ungearitems-plugin` on the authority node.
#[derive(Clone, Debug, Message)]
pub struct RequestCraftRepellent {
    pub ghost_type: GhostType,
}

/// Sent from a pure join client to the dedicated server every frame that local
/// repellent particles register hits against the ghost. The server applies the
/// accumulated frame damage to the authoritative `GhostSprite` and replicates
/// the result back to all clients so squish/stretch visuals work correctly.
#[derive(Clone, Debug, Serialize, Deserialize, Message)]
pub struct RepellentHitNetMessage {
    pub hits_this_frame: f32,
    pub misses_this_frame: f32,
}

/// Emitted locally on a player-bearing node when the repellent flask starts
/// dispensing (i.e. a full flask begins its discharge). Used by the summary
/// domain to count repellent uses without the gear domain importing presentation
/// types.
#[derive(Clone, Debug, Message)]
pub struct RepellentUsedEvent;
