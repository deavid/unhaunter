use crate::gear_stuff::GearStuff;
use crate::gear_usable::GearUsable;
use bevy::prelude::*;
use uncore::types::gear_kind::GearKind;
use uncore::types::ghost::types::GhostType;
use uncore::{
    components::board::position::Position,
    types::gear::{equipmentposition::EquipmentPosition, spriteid::GearSpriteID},
};

/// A wrapper struct for holding a `GearKind`.
#[derive(Debug, Default, Clone)]
pub struct Gear {
    /// The type of gear being held.
    pub kind: GearKind,
    pub data: Option<Box<dyn GearUsable>>,
}

impl Gear {
    /// Creates a new empty Gear
    pub fn none() -> Self {
        Self {
            kind: GearKind::None,
            data: None,
        }
    }

    /// Creates a new Gear of the specified Kind
    pub fn new_from_kind(kind: GearKind, data: Box<dyn GearUsable>) -> Self {
        Self {
            kind,
            data: Some(data),
        }
    }

    /// Takes the content of the current Gear and returns it, leaving None.
    pub fn take(&mut self) -> Self {
        let mut new = Self::none();
        std::mem::swap(&mut new, self);
        new
    }
}

impl GearUsable for Gear {
    fn get_display_name(&self) -> &'static str {
        match &self.data {
            Some(x) => x.get_display_name(),
            None => "",
        }
    }

    fn get_description(&self) -> &'static str {
        match &self.data {
            Some(x) => x.get_description(),
            None => "",
        }
    }

    fn get_status(&self) -> String {
        match &self.data {
            Some(x) => x.get_status(),
            None => "".to_string(),
        }
    }

    fn set_trigger(&mut self, gs: &mut GearStuff) {
        let sound_file = "sounds/switch-on-1.ogg";
        gs.play_audio_nopos(sound_file.into(), 0.6);
        let ni = |s| warn!("Trigger not implemented for {:?}", s);
        match &mut self.data {
            Some(x) => x.set_trigger(gs),
            None => ni(&self),
        }
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        match &self.data {
            Some(x) => x.get_sprite_idx(),
            None => GearSpriteID::None,
        }
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        if let Some(data) = &self.data {
            data.box_clone()
        } else {
            Box::new(Gear::none())
        }
    }

    fn update(&mut self, gs: &mut GearStuff, pos: &Position, ep: &EquipmentPosition) {
        if let Some(x) = &mut self.data {
            x.update(gs, pos, ep)
        }
    }

    fn needs_darkness(&self) -> bool {
        match &self.data {
            Some(x) => x.needs_darkness(),
            None => false,
        }
    }

    fn is_enabled(&self) -> bool {
        match &self.data {
            Some(x) => x.is_enabled(),
            None => false,
        }
    }

    fn power(&self) -> f32 {
        match &self.data {
            Some(x) => x.power(),
            None => 0.0,
        }
    }

    fn color(&self) -> Color {
        match &self.data {
            Some(x) => x.color(),
            None => Color::WHITE,
        }
    }

    fn can_fill_liquid(&self, ghost_type: GhostType) -> bool {
        match &self.data {
            Some(x) => x.can_fill_liquid(ghost_type),
            None => false,
        }
    }

    fn do_fill_liquid(&mut self, ghost_type: GhostType) -> bool {
        match &mut self.data {
            Some(x) => x.do_fill_liquid(ghost_type),
            None => {
                warn!("do_fill_liquid called on empty Gear");
                false
            }
        }
    }

    fn is_electronic(&self) -> bool {
        match &self.data {
            Some(x) => x.is_electronic(),
            None => false,
        }
    }

    fn can_enable(&self) -> bool {
        match &self.data {
            Some(x) => x.can_enable(),
            None => false,
        }
    }

    fn is_status_text_showing_evidence(&self) -> f32 {
        match &self.data {
            Some(x) => x.is_status_text_showing_evidence(),
            None => 0.0,
        }
    }

    fn is_icon_showing_evidence(&self) -> f32 {
        match &self.data {
            Some(x) => x.is_icon_showing_evidence(),
            None => 0.0,
        }
    }

    fn is_sound_showing_evidence(&self) -> f32 {
        match &self.data {
            Some(x) => x.is_sound_showing_evidence(),
            None => 0.0,
        }
    }

    fn is_blinking_hint_active(&self) -> bool {
        match &self.data {
            Some(x) => x.is_blinking_hint_active(),
            None => false,
        }
    }
}
