pub mod kind;
pub mod utils;

pub use kind::{GearKind, PlayerGearKind};
pub use unfoundation_core::types::gear::{EquipmentPosition, Hand, VisualKey};
pub type SpriteID = VisualKey;
