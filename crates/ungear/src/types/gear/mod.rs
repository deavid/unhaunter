pub mod utils;

pub use uncore_foundation::types::gear::{EquipmentPosition, GearKind, GearSpriteID, Hand};
pub type SpriteID = GearSpriteID;

use crate::gear_stuff::GearStuff;
use crate::gear_usable::GearUsable;
use unspatial::Position;

#[derive(Debug)]
pub struct Gear {
    pub kind: GearKind,
    pub gear: Box<dyn GearUsable>,
}

impl Clone for Gear {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            gear: self.gear.box_clone(),
        }
    }
}

impl Default for Gear {
    fn default() -> Self {
        Self {
            kind: GearKind::None,
            gear: Box::new(NoneGear),
        }
    }
}

impl Gear {
    pub fn new(kind: GearKind, gear: Box<dyn GearUsable>) -> Self {
        Self { kind, gear }
    }
    pub fn new_from_kind(kind: GearKind, gear: Box<dyn GearUsable>) -> Self {
        Self { kind, gear }
    }
    pub fn none() -> Self {
        Self::default()
    }
    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
    pub fn update(&mut self, gs: &mut GearStuff, pos: &Position, ep: &EquipmentPosition) {
        self.gear.update(gs, pos, ep);
    }
    pub fn get_sprite_idx(&self) -> GearSpriteID {
        self.gear.get_sprite_idx()
    }
    pub fn get_display_name(&self) -> &'static str {
        self.gear.get_display_name()
    }
    pub fn get_description(&self) -> &'static str {
        self.gear.get_description()
    }
    pub fn get_status(&self) -> String {
        self.gear.get_status()
    }
    pub fn set_trigger(&mut self, gs: &mut GearStuff) {
        self.gear.set_trigger(gs);
    }
    pub fn is_enabled(&self) -> bool {
        self.gear.is_enabled()
    }
    pub fn can_enable(&self) -> bool {
        self.gear.can_enable()
    }
    pub fn needs_darkness(&self) -> bool {
        self.gear.needs_darkness()
    }
    pub fn is_status_text_showing_evidence(&self) -> f32 {
        self.gear.is_status_text_showing_evidence()
    }
    pub fn is_icon_showing_evidence(&self) -> f32 {
        self.gear.is_icon_showing_evidence()
    }
    pub fn is_sound_showing_evidence(&self) -> f32 {
        self.gear.is_sound_showing_evidence()
    }
    pub fn is_blinking_hint_active(&self) -> bool {
        self.gear.is_blinking_hint_active()
    }
}

#[derive(Debug, Clone)]
pub struct NoneGear;

impl GearUsable for NoneGear {
    fn get_display_name(&self) -> &'static str {
        "None"
    }

    fn get_description(&self) -> &'static str {
        ""
    }

    fn get_status(&self) -> String {
        "".to_string()
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {}

    fn get_sprite_idx(&self) -> GearSpriteID {
        GearSpriteID::None
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}
