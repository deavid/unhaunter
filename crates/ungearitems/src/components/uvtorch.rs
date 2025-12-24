use super::{EquipmentPosition, Gear, GearKind, GearSpriteID, on_off};
use bevy::prelude::*;
use rand::Rng;
use uncore_components::{Battery, Electronic, GearSprite, ItemName, StatusText, Toggleable};
use uncore_foundation::random_seed;
use ungear::gear_stuff::GearStuff;
use ungear::gear_usable::GearUsable;
use unspatial::Position;

#[derive(Component, Debug, Clone)]
pub struct UVTorch {
    pub enabled: bool,
    pub output_power: f32,
}

impl Default for UVTorch {
    fn default() -> Self {
        Self {
            enabled: false,
            output_power: 0.0,
        }
    }
}

impl UVTorch {
    pub fn calculate_output_power(&self, battery_level: f32, glitch_timer: f32) -> f32 {
        if glitch_timer > 0.0 {
            return glitch_timer * 0.5; // Weaker flickering than flashlight
        }

        match self.enabled {
            false => 0.0,
            true => 2.0 * (battery_level.sqrt() + 0.05),
        }
    }

    pub fn update_output_power(&mut self, battery_level: f32, glitch_timer: f32) {
        let new_power = self.calculate_output_power(battery_level, glitch_timer);
        self.output_power = (self.output_power * 10.0 + new_power) / 11.0;
    }

    pub fn is_enabled(&self, battery_level: f32, glitch_timer: f32) -> bool {
        self.enabled && battery_level > 0.0 && glitch_timer <= 0.01
    }

    pub fn can_enable(&self, battery_level: f32, glitch_timer: f32) -> bool {
        battery_level > 0.0 && glitch_timer <= 0.01
    }
}

impl GearUsable for UVTorch {
    fn get_display_name(&self) -> &'static str {
        "UV Torch"
    }

    fn get_description(&self) -> &'static str {
        "A ultraviolet flashlight. Can reveal fingerprints and other UV-reactive evidence."
    }

    fn get_status(&self) -> String {
        on_off(self.enabled).to_string()
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {}

    fn get_sprite_idx(&self) -> GearSpriteID {
        if self.enabled {
            GearSpriteID::UVTorchOn
        } else {
            GearSpriteID::UVTorchOff
        }
    }

    fn update(&mut self, _gs: &mut GearStuff, _pos: &Position, _ep: &EquipmentPosition) {}

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<UVTorch> for Gear {
    fn from(value: UVTorch) -> Self {
        Gear::new_from_kind(GearKind::UVTorch, value.box_clone())
    }
}

pub fn update_uvtorch(
    mut q_uvtorch: Query<(
        &mut UVTorch,
        &mut StatusText,
        &mut GearSprite,
        &Toggleable,
        &mut Battery,
        &mut Electronic,
        &Position,
        &ItemName,
    )>,
    mut gs: GearStuff,
) {
    for (mut uvtorch, mut status, mut sprite, toggle, mut battery, electronic, pos, name) in
        q_uvtorch.iter_mut()
    {
        // Sync internal enabled with Toggleable
        uvtorch.enabled = toggle.is_on;

        // Update Battery Drain Rate
        battery.drain_rate = if uvtorch.enabled { 0.0001 } else { 0.0 };

        // Play static/interference sounds when glitching
        if electronic.glitch_timer > 0.0
            && uvtorch.enabled
            && random_seed::rng().random_range(0.0..1.0) < 0.2
        {
            gs.play_audio("sounds/effects-chirp-short.ogg".into(), 0.3, pos);
        }

        uvtorch.update_output_power(battery.level, electronic.glitch_timer);

        // Update Sprite
        sprite.0 = if electronic.glitch_timer > 0.0 {
            if random_seed::rng().random_range(0.0..1.0) < 0.3 {
                GearSpriteID::UVTorchOff
            } else {
                GearSpriteID::UVTorchOn
            }
        } else if uvtorch.enabled {
            GearSpriteID::UVTorchOn
        } else {
            GearSpriteID::UVTorchOff
        };

        // Update Status Text
        let on_s = on_off(uvtorch.enabled);
        if uvtorch.enabled && electronic.glitch_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..4) {
                0 => "Bat---y: E--OR",
                1 => "UV Status: -.--%",
                2 => "WAVEL--GTH FA--T",
                _ => "CALIB---ING...",
            };
            status.0 = format!("{}: {}\n{}", name.0, on_s, garbled);
        } else {
            let msg = if uvtorch.enabled {
                format!("Battery: {:>3.0}%", battery.level * 100.0)
            } else {
                "".to_string()
            };
            status.0 = format!("{}: {}\n{}", name.0, on_s, msg);
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, update_uvtorch);
}
