use uncore::random_seed;
use uncore::{
    components::board::position::Position, types::gear::equipmentposition::EquipmentPosition,
};
use ungear::gear_stuff::GearStuff;

use super::{Gear, GearKind, GearSpriteID, GearUsable};
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
    pub battery_level: f32,
    pub frame_counter: u8,
    pub rand: u8,
    pub flicker_timer: f32,
    pub output_power: f32,
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
            flicker_timer: 0.0,
            output_power: 0.0,
        }
    }
}

impl Flashlight {
    pub fn calculate_output_power(&self) -> f32 {
        if self.flicker_timer > 0.0 {
            return self.flicker_timer * 4.0;
        }
        let bat = self.battery_level.sqrt() + 0.02;
        let pow = match self.status {
            FlashlightStatus::Off => 0.0,
            FlashlightStatus::Low => 4.0,
            FlashlightStatus::Mid => 16.0,
            FlashlightStatus::High => 64.0,
        };
        pow * bat
    }
    pub fn update_output_power(&mut self) {
        let new_power = self.calculate_output_power();

        self.output_power = (self.output_power * 2.0 + new_power) / 3.0;
    }

    fn can_enable_status(&self, target_status: FlashlightStatus) -> bool {
        if target_status == FlashlightStatus::Off {
            return true; // Can always turn off
        }
        self.battery_level > 0.0 && self.inner_temp <= 1.0 && self.flicker_timer <= 0.0
    }
}

impl GearUsable for Flashlight {
    fn update(&mut self, gs: &mut GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        self.frame_counter += 1;
        self.frame_counter %= 210;
        if self.frame_counter % 5 == 0 {
            self.rand = random_seed::rng().random_range(0..12);
            const HS_MASS: f32 = 2.0;
            self.heatsink_temp = (self.heatsink_temp * HS_MASS + self.inner_temp) / (HS_MASS + 1.0);
        }

        // --- EMI Effects ---
        // Apply EMI if warning is active and we're electronic
        if let Some(ghost_pos) = &gs.bf.ghost_warning_position {
            let distance2 = pos.distance2(ghost_pos);
            self.apply_electromagnetic_interference(gs.bf.ghost_warning_intensity, distance2);
        }
        // --- End EMI Effects ---

        // Normal behavior only if there's no EMI or it's not significant
        if self.flicker_timer <= 0.0 {
            self.battery_level -= self.power() / 500000.0;
            if self.battery_level < 0.0 {
                self.battery_level = 0.0;
                self.status = FlashlightStatus::Off;
            }
            self.inner_temp += self.power() / 50000.0;
            self.inner_temp /= 1.00032;
            if self.inner_temp > 1.0 && self.status != FlashlightStatus::Off {
                self.status = FlashlightStatus::Off;
                gs.play_audio("sounds/effects-dingdingding.ogg".into(), 0.7, pos);
            }
        } else {
            self.flicker_timer -= gs.time.delta_secs();
            if self.flicker_timer < 0.0 && self.status == FlashlightStatus::Off {
                self.status = FlashlightStatus::Low;
                self.flicker_timer = 0.0;
            }
        }
        self.update_output_power();
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        if self.flicker_timer > 0.0 {
            return GearSpriteID::Flashlight3;
        }
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

        // Show garbled text when glitching
        if self.flicker_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..4) {
                0 => "Bat---y: E--OR",
                1 => "UV Status: -.--%",
                2 => "WAVEL--GTH FA--T",
                _ => "CALIB---ING...",
            };
            return format!("{name}: {on_s}  {overheat}\n{garbled}");
        }

        let heat_temp = 15.0 + self.heatsink_temp * 70.0;
        format!(
            "{name}: {on_s}  {overheat}\nBattery:   {:>3.0}% {heat_temp:>5.1}ºC",
            self.battery_level * 100.0
        )
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {
        if self.flicker_timer <= 0.0 {
            let next_status = self.status.next().unwrap_or_default();
            if self.can_enable_status(next_status.clone()) {
                self.status = next_status;
            } else if self.status != FlashlightStatus::Off {
                // If it can't be enabled to the next state but is on, turn it off
                self.status = FlashlightStatus::Off;
            }
        }
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn power(&self) -> f32 {
        self.output_power
    }

    fn color(&self) -> Color {
        if self.flicker_timer > 0.0 {
            return Color::srgb(0.85, 0.92, 1.0);
        }
        // Beige
        Color::srgb(0.96, 0.92, 0.82)
    }

    fn apply_electromagnetic_interference(&mut self, warning_level: f32, distance2: f32) {
        if warning_level < 0.0001 || self.status == FlashlightStatus::Off {
            return;
        }
        let mut rng = random_seed::rng();

        // Scale effect by distance and warning level
        let effect_strength = warning_level * (100.0 / distance2).min(1.0);

        // Random EMF spikes
        if rng.random_range(0.0..1.0) < effect_strength.powi(2) && self.flicker_timer < 0.1 {
            // Jumble numbers temporarily
            self.flicker_timer = rng.random_range(0.3..0.8);
            self.status = FlashlightStatus::Off;
        }
    }

    fn is_electronic(&self) -> bool {
        true
    }

    fn is_enabled(&self) -> bool {
        self.status != FlashlightStatus::Off
            && self.battery_level > 0.0
            && self.inner_temp <= 1.0
            && self.flicker_timer <= 0.0
    }

    fn can_enable(&self) -> bool {
        // This method now considers if any ON state can be achieved.
        // For Flashlight, it can always be turned to Off,
        // but to turn it ON (Low, Mid, High) it needs battery and not be overheated or flickering.
        self.can_enable_status(FlashlightStatus::Low)
            || self.can_enable_status(FlashlightStatus::Mid)
            || self.can_enable_status(FlashlightStatus::High)
    }
}

impl From<Flashlight> for Gear {
    fn from(value: Flashlight) -> Self {
        Gear::new_from_kind(GearKind::Flashlight, value.box_clone())
    }
}
