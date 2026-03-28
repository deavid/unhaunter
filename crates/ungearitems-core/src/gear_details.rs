use crate::components::flashlight::FlashlightStatus;
use serde::{Deserialize, Serialize};
use uninvestigation_core::ghost::GhostType;

/// A compact summary of a gear item's current state, suitable for network serialization.
///
/// Used by `PlayerInput` to communicate which gear item a player is currently
/// interacting with and what that item's state is.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GearDetails {
    Flashlight(FlashlightStatus),
    Thermometer {
        temp: f32,
    },
    EMF {
        level: f32,
    },
    Sage {
        consumed: bool,
        is_active: bool,
        remaining_secs: f32,
    },
    RepellentFlask {
        qty: i32,
        active: bool,
        liquid_content: Option<GhostType>,
    },
    SpiritBox {
        charge: f32,
        ghost_answer: bool,
    },
    None,
}
