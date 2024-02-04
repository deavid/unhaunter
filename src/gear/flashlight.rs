use bevy::prelude::*;
use enum_iterator::Sequence;
use rand::Rng;

use super::{GearSpriteID, GearUsable};

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
            FlashlightStatus::High => "HI",
        }
    }
}

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct Flashlight {
    pub status: FlashlightStatus,
    pub frame_counter: u8,
    pub rand: u8,
}

impl GearUsable for Flashlight {
    fn update(&mut self) {
        self.frame_counter += 1;
        self.frame_counter %= 210;
        if self.frame_counter % 5 == 0 {
            self.rand = rand::thread_rng().gen_range(0..12);
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

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = self.status.string();
        format!("{name}: {on_s}")
    }

    fn set_trigger(&mut self) {
        self.status = self.status.next().unwrap_or_default();
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}
