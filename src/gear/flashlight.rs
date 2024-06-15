use bevy::prelude::*;
use enum_iterator::Sequence;
use rand::Rng;

use crate::board::Position;

use super::{playergear::EquipmentPosition, Gear, GearKind, GearSpriteID, GearUsable};

#[derive(Debug, Clone, Default, PartialEq, Eq, Sequence)]
pub enum FlashlightStatus {
    #[default]
    Off,
    Low,
    Mid,
    High,
}

impl FlashlightStatus {
    pub fn string(&self) -> &'static str {
        match self {
            FlashlightStatus::Off => "OFF",
            FlashlightStatus::Low => "LOW",
            FlashlightStatus::Mid => "MID",
            FlashlightStatus::High => " HI",
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Flashlight {
    pub status: FlashlightStatus,
    pub inner_temp: f32,
    pub heatsink_temp: f32,
    pub battery_level: f32,
    pub frame_counter: u8,
    pub rand: u8,
}

impl Default for Flashlight {
    fn default() -> Self {
        Self {
            status: Default::default(),
            inner_temp: Default::default(),
            heatsink_temp: Default::default(),
            battery_level: 1.0,
            frame_counter: Default::default(),
            rand: Default::default(),
        }
    }
}

impl Flashlight {
    pub fn power(&self) -> f32 {
        let bat = self.battery_level.sqrt() + 0.02;
        let pow = match self.status {
            FlashlightStatus::Off => 0.0,
            FlashlightStatus::Low => 2.0,
            FlashlightStatus::Mid => 6.0,
            FlashlightStatus::High => 10.0,
        };
        pow * bat
    }
    pub fn color(&self) -> Color {
        // Beige
        Color::rgb(0.96, 0.92, 0.82)
    }
}

impl GearUsable for Flashlight {
    fn update(&mut self, gs: &mut super::GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        self.frame_counter += 1;
        self.frame_counter %= 210;
        if self.frame_counter % 5 == 0 {
            self.rand = rand::thread_rng().gen_range(0..12);
        }
        self.battery_level -= self.power() / 500000.0;
        if self.battery_level < 0.0 {
            self.battery_level = 0.0;
            self.status = FlashlightStatus::Off;
        }
        self.inner_temp += self.power() / 10000.0;
        self.inner_temp /= 1.0006;

        const HS_MASS: f32 = 200.0;
        self.heatsink_temp = (self.heatsink_temp * HS_MASS + self.inner_temp) / (HS_MASS + 1.0);
        if self.heatsink_temp > 1.0 && self.status == FlashlightStatus::High {
            self.status = FlashlightStatus::Mid;
            gs.play_audio("sounds/effects-dingdingding.ogg".into(), 0.4, pos);
        } else if self.heatsink_temp > 1.2 && self.status == FlashlightStatus::Mid {
            self.status = FlashlightStatus::Low;
            gs.play_audio("sounds/effects-dingdingding.ogg".into(), 0.4, pos);
        }
    }
    fn get_sprite_idx(&self) -> GearSpriteID {
        if self.rand == 0 {
            match self.status {
                FlashlightStatus::Off => GearSpriteID::FlashlightOff,
                FlashlightStatus::Low => GearSpriteID::Flashlight2,
                FlashlightStatus::Mid => GearSpriteID::Flashlight1,
                FlashlightStatus::High => GearSpriteID::Flashlight2,
            }
        } else {
            match self.status {
                FlashlightStatus::Off => GearSpriteID::FlashlightOff,
                FlashlightStatus::Low => GearSpriteID::Flashlight1,
                FlashlightStatus::Mid => GearSpriteID::Flashlight2,
                FlashlightStatus::High => GearSpriteID::Flashlight3,
            }
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Flashlight"
    }

    fn get_description(&self) -> &'static str {
        "Iluminates the way. Imprescindible tool to work in the dark."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = self.status.string();
        let overheat = if self.heatsink_temp > 0.8 {
            "OVERHEAT"
        } else {
            ""
        };
        let heat_temp = 15.0 + self.heatsink_temp * 70.0;

        format!(
            "{name}: {on_s}  {overheat}\nBattery:   {:>3.0}% {heat_temp:>5.1}ºC",
            self.battery_level * 100.0
        )
    }

    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        self.status = self.status.next().unwrap_or_default();
    }

    fn _box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<Flashlight> for Gear {
    fn from(value: Flashlight) -> Self {
        Gear::new_from_kind(GearKind::Flashlight(value))
    }
}
