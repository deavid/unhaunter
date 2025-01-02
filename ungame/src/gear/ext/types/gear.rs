use bevy::prelude::*;

use crate::gear::ext::systemparam::gearstuff::GearStuff;
use uncore::{
    components::board::position::Position,
    types::gear::{equipmentposition::EquipmentPosition, spriteid::GearSpriteID},
};

use super::{gearkind::GearKind, traits::GearUsable};

/// A wrapper struct for holding a `GearKind`.
#[derive(Debug, Default, Clone)]
pub struct Gear {
    /// The type of gear being held.
    pub kind: GearKind,
}

impl Gear {
    /// Creates a new empty Gear
    pub fn none() -> Self {
        Self {
            kind: GearKind::None,
        }
    }

    /// Creates a new Gear of the specified Kind
    pub fn new_from_kind(kind: GearKind) -> Self {
        Self { kind }
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
        match &self.kind {
            GearKind::Thermometer(x) => x.get_display_name(),
            GearKind::Flashlight(x) => x.get_display_name(),
            GearKind::EMFMeter(x) => x.get_display_name(),
            GearKind::Recorder(x) => x.get_display_name(),
            GearKind::GeigerCounter(x) => x.get_display_name(),
            GearKind::UVTorch(x) => x.get_display_name(),
            GearKind::IonMeter(x) => x.get_display_name(),
            GearKind::Photocam(x) => x.get_display_name(),
            GearKind::SpiritBox(x) => x.get_display_name(),
            GearKind::RedTorch(x) => x.get_display_name(),
            GearKind::Compass(x) => x.get_display_name(),
            GearKind::ThermalImager(x) => x.get_display_name(),
            GearKind::EStaticMeter(x) => x.get_display_name(),
            GearKind::Videocam(x) => x.get_display_name(),
            GearKind::MotionSensor(x) => x.get_display_name(),
            GearKind::RepellentFlask(x) => x.get_display_name(),
            GearKind::QuartzStone(x) => x.get_display_name(),
            GearKind::Salt(x) => x.get_display_name(),
            GearKind::SageBundle(x) => x.get_display_name(),
            GearKind::None => "",
        }
    }

    fn get_description(&self) -> &'static str {
        match &self.kind {
            GearKind::Thermometer(x) => x.get_description(),
            GearKind::Flashlight(x) => x.get_description(),
            GearKind::EMFMeter(x) => x.get_description(),
            GearKind::Recorder(x) => x.get_description(),
            GearKind::GeigerCounter(x) => x.get_description(),
            GearKind::UVTorch(x) => x.get_description(),
            GearKind::IonMeter(x) => x.get_description(),
            GearKind::Photocam(x) => x.get_description(),
            GearKind::SpiritBox(x) => x.get_description(),
            GearKind::RedTorch(x) => x.get_description(),
            GearKind::Compass(x) => x.get_description(),
            GearKind::ThermalImager(x) => x.get_description(),
            GearKind::EStaticMeter(x) => x.get_description(),
            GearKind::Videocam(x) => x.get_description(),
            GearKind::MotionSensor(x) => x.get_description(),
            GearKind::RepellentFlask(x) => x.get_description(),
            GearKind::QuartzStone(x) => x.get_description(),
            GearKind::Salt(x) => x.get_description(),
            GearKind::SageBundle(x) => x.get_description(),
            GearKind::None => "",
        }
    }

    fn get_status(&self) -> String {
        match &self.kind {
            GearKind::Thermometer(x) => x.get_status(),
            GearKind::Flashlight(x) => x.get_status(),
            GearKind::EMFMeter(x) => x.get_status(),
            GearKind::Recorder(x) => x.get_status(),
            GearKind::GeigerCounter(x) => x.get_status(),
            GearKind::UVTorch(x) => x.get_status(),
            GearKind::IonMeter(x) => x.get_status(),
            GearKind::Photocam(x) => x.get_status(),
            GearKind::SpiritBox(x) => x.get_status(),
            GearKind::RedTorch(x) => x.get_status(),
            GearKind::Compass(x) => x.get_status(),
            GearKind::ThermalImager(x) => x.get_status(),
            GearKind::EStaticMeter(x) => x.get_status(),
            GearKind::Videocam(x) => x.get_status(),
            GearKind::MotionSensor(x) => x.get_status(),
            GearKind::RepellentFlask(x) => x.get_status(),
            GearKind::QuartzStone(x) => x.get_status(),
            GearKind::Salt(x) => x.get_status(),
            GearKind::SageBundle(x) => x.get_status(),
            GearKind::None => "".to_string(),
        }
    }

    fn set_trigger(&mut self, gs: &mut GearStuff) {
        let sound_file = "sounds/switch-on-1.ogg";
        gs.play_audio_nopos(sound_file.into(), 0.6);
        let ni = |s| warn!("Trigger not implemented for {:?}", s);
        match &mut self.kind {
            GearKind::Thermometer(x) => x.set_trigger(gs),
            GearKind::Flashlight(x) => x.set_trigger(gs),
            GearKind::ThermalImager(x) => x.set_trigger(gs),
            GearKind::EMFMeter(x) => x.set_trigger(gs),
            GearKind::Recorder(x) => x.set_trigger(gs),
            GearKind::GeigerCounter(x) => x.set_trigger(gs),
            GearKind::RedTorch(x) => x.set_trigger(gs),
            GearKind::UVTorch(x) => x.set_trigger(gs),
            GearKind::Photocam(x) => x.set_trigger(gs),
            GearKind::IonMeter(x) => x.set_trigger(gs),
            GearKind::SpiritBox(x) => x.set_trigger(gs),
            GearKind::Compass(x) => x.set_trigger(gs),
            GearKind::EStaticMeter(x) => x.set_trigger(gs),
            GearKind::Videocam(x) => x.set_trigger(gs),
            GearKind::MotionSensor(x) => x.set_trigger(gs),
            GearKind::RepellentFlask(x) => x.set_trigger(gs),
            GearKind::QuartzStone(x) => x.set_trigger(gs),
            GearKind::Salt(x) => x.set_trigger(gs),
            GearKind::SageBundle(x) => x.set_trigger(gs),
            GearKind::None => ni(&self),
        }
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        match &self.kind {
            GearKind::Thermometer(x) => x.get_sprite_idx(),
            GearKind::Flashlight(x) => x.get_sprite_idx(),
            GearKind::ThermalImager(x) => x.get_sprite_idx(),
            GearKind::EMFMeter(x) => x.get_sprite_idx(),
            GearKind::Recorder(x) => x.get_sprite_idx(),
            GearKind::GeigerCounter(x) => x.get_sprite_idx(),
            GearKind::RedTorch(x) => x.get_sprite_idx(),
            GearKind::UVTorch(x) => x.get_sprite_idx(),
            GearKind::Photocam(x) => x.get_sprite_idx(),
            GearKind::IonMeter(x) => x.get_sprite_idx(),
            GearKind::SpiritBox(x) => x.get_sprite_idx(),
            GearKind::Compass(x) => x.get_sprite_idx(),
            GearKind::EStaticMeter(x) => x.get_sprite_idx(),
            GearKind::Videocam(x) => x.get_sprite_idx(),
            GearKind::MotionSensor(x) => x.get_sprite_idx(),
            GearKind::RepellentFlask(x) => x.get_sprite_idx(),
            GearKind::QuartzStone(x) => x.get_sprite_idx(),
            GearKind::Salt(x) => x.get_sprite_idx(),
            GearKind::SageBundle(x) => x.get_sprite_idx(),
            GearKind::None => GearSpriteID::None,
        }
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        unimplemented!();
    }

    fn update(&mut self, gs: &mut GearStuff, pos: &Position, ep: &EquipmentPosition) {
        match &mut self.kind {
            GearKind::Thermometer(x) => x.update(gs, pos, ep),
            GearKind::Flashlight(x) => x.update(gs, pos, ep),
            GearKind::ThermalImager(x) => x.update(gs, pos, ep),
            GearKind::EMFMeter(x) => x.update(gs, pos, ep),
            GearKind::Recorder(x) => x.update(gs, pos, ep),
            GearKind::GeigerCounter(x) => x.update(gs, pos, ep),
            GearKind::RedTorch(x) => x.update(gs, pos, ep),
            GearKind::UVTorch(x) => x.update(gs, pos, ep),
            GearKind::Photocam(x) => x.update(gs, pos, ep),
            GearKind::IonMeter(x) => x.update(gs, pos, ep),
            GearKind::SpiritBox(x) => x.update(gs, pos, ep),
            GearKind::Compass(x) => x.update(gs, pos, ep),
            GearKind::EStaticMeter(x) => x.update(gs, pos, ep),
            GearKind::Videocam(x) => x.update(gs, pos, ep),
            GearKind::MotionSensor(x) => x.update(gs, pos, ep),
            GearKind::RepellentFlask(x) => x.update(gs, pos, ep),
            GearKind::QuartzStone(x) => x.update(gs, pos, ep),
            GearKind::Salt(x) => x.update(gs, pos, ep),
            GearKind::SageBundle(x) => x.update(gs, pos, ep),
            GearKind::None => {}
        }
    }
}
