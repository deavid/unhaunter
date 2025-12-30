use uncore_components::{
    Battery, Electronic, GearSprite, ItemName, LightEmitter, StatusText, Toggleable, Triggered,
};
use uncore_foundation::random_seed;
use ungear::gear_stuff::GearStuff;
use unspatial_core::Position;

use super::GearSpriteID;
use bevy::prelude::*;
use enum_iterator::Sequence;
use rand::Rng;

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
    pub frame_counter: u8,
    pub rand: u8,
    pub output_power: f32,
}

impl Default for Flashlight {
    fn default() -> Self {
        Self {
            status: Default::default(),
            inner_temp: Default::default(),
            heatsink_temp: Default::default(),
            frame_counter: Default::default(),
            rand: Default::default(),
            output_power: 0.0,
        }
    }
}

impl Flashlight {
    pub fn calculate_output_power(&self) -> f32 {
        match self.status {
            FlashlightStatus::Off => 0.0,
            FlashlightStatus::Low => 4.0,
            FlashlightStatus::Mid => 16.0,
            FlashlightStatus::High => 64.0,
        }
    }
    pub fn update_output_power(&mut self, battery_level: f32, glitch_timer: f32) {
        let mut new_power = self.calculate_output_power();
        if glitch_timer > 0.0 {
            new_power = glitch_timer * 4.0;
        }
        let bat = battery_level.sqrt() + 0.02;
        new_power *= bat;

        self.output_power = (self.output_power * 2.0 + new_power) / 3.0;
    }

    fn can_enable_status(&self, target_status: FlashlightStatus, battery_level: f32) -> bool {
        if target_status == FlashlightStatus::Off {
            return true; // Can always turn off
        }
        battery_level > 0.0 && self.inner_temp <= 1.0
    }
}

pub fn update_flashlight(
    mut q_flashlight: Query<(
        &mut Flashlight,
        &mut LightEmitter,
        &mut StatusText,
        &mut GearSprite,
        &mut Toggleable,
        &mut Battery,
        &mut Electronic,
        Option<&Triggered>,
        &Position,
        &ItemName,
    )>,
    mut gs: GearStuff,
) {
    for (
        mut flashlight,
        mut flashlight_render,
        mut status,
        mut sprite,
        mut toggle,
        mut battery,
        electronic,
        triggered,
        pos,
        name,
    ) in q_flashlight.iter_mut()
    {
        // Handle Trigger
        if triggered.is_some() && electronic.glitch_timer <= 0.0 {
            let next_status = flashlight.status.next().unwrap_or_default();
            if flashlight.can_enable_status(next_status.clone(), battery.level) {
                flashlight.status = next_status;
            } else if flashlight.status != FlashlightStatus::Off {
                flashlight.status = FlashlightStatus::Off;
            }
        }

        // Sync Toggleable with FlashlightStatus
        toggle.is_on = flashlight.status != FlashlightStatus::Off;

        // Update Logic
        flashlight.frame_counter += 1;
        flashlight.frame_counter %= 210;
        if flashlight.frame_counter.is_multiple_of(5) {
            flashlight.rand = random_seed::rng().random_range(0..12);
            const HS_MASS: f32 = 2.0;
            flashlight.heatsink_temp =
                (flashlight.heatsink_temp * HS_MASS + flashlight.inner_temp) / (HS_MASS + 1.0);
        }

        // Update Battery Drain Rate
        battery.drain_rate = flashlight.calculate_output_power() / 5000.0;

        if electronic.glitch_timer <= 0.0 {
            if battery.level <= 0.0 {
                flashlight.status = FlashlightStatus::Off;
            }
            flashlight.inner_temp += flashlight.output_power / 50000.0;
            flashlight.inner_temp /= 1.00032;
            if flashlight.inner_temp > 1.0 && flashlight.status != FlashlightStatus::Off {
                flashlight.status = FlashlightStatus::Off;
                gs.play_audio("sounds/effects-dingdingding.ogg".into(), 0.7, pos);
            }
        } else if flashlight.status != FlashlightStatus::Off {
            // If it was on and it's glitching, turn it off temporarily
            flashlight.status = FlashlightStatus::Off;
        } else if electronic.glitch_timer < 0.01 {
            // If it was off due to glitching and glitch is ending, turn it back on
            flashlight.status = FlashlightStatus::Low;
        }

        flashlight.update_output_power(battery.level, electronic.glitch_timer);

        // Sync with Render Component
        flashlight_render.power = flashlight.output_power;

        // Update Sprite
        sprite.0 = if electronic.glitch_timer > 0.0 {
            GearSpriteID::Flashlight3
        } else if flashlight.rand == 0 {
            match flashlight.status {
                FlashlightStatus::Off => GearSpriteID::FlashlightOff,
                FlashlightStatus::Low => GearSpriteID::Flashlight2,
                FlashlightStatus::Mid => GearSpriteID::Flashlight1,
                FlashlightStatus::High => GearSpriteID::Flashlight2,
            }
        } else {
            match flashlight.status {
                FlashlightStatus::Off => GearSpriteID::FlashlightOff,
                FlashlightStatus::Low => GearSpriteID::Flashlight1,
                FlashlightStatus::Mid => GearSpriteID::Flashlight2,
                FlashlightStatus::High => GearSpriteID::Flashlight3,
            }
        };

        // Update Status Text
        let on_s = flashlight.status.string();
        let overheat = if flashlight.heatsink_temp > 0.8 {
            "OVERHEAT"
        } else {
            ""
        };

        if electronic.glitch_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..4) {
                0 => "Bat---y: E--OR",
                1 => "UV Status: -.--%",
                2 => "WAVEL--GTH FA--T",
                _ => "CALIB---ING...",
            };
            status.0 = format!("{}: {}  {}\n{}", name.0, on_s, overheat, garbled);
        } else {
            let heat_temp = 15.0 + flashlight.heatsink_temp * 70.0;
            status.0 = format!(
                "{}: {}  {}\nBattery:   {:>3.0}% {:>5.1}ºC",
                name.0,
                on_s,
                overheat,
                battery.level * 100.0,
                heat_temp
            );
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, update_flashlight);
}
