use super::{Gear, GearKind, GearSpriteID, GearUsable, on_off};
use bevy::prelude::*;
use rand::Rng;
use uncore::components::board::position::Position;
use uncore::random_seed;
use uncore::types::gear::equipmentposition::EquipmentPosition;

#[derive(Component, Debug, Clone)]
pub struct UVTorch {
    pub enabled: bool,
    pub display_glitch_timer: f32,
    pub output_power: f32,
    pub battery_level: f32,
}

impl Default for UVTorch {
    fn default() -> Self {
        Self {
            enabled: false,
            display_glitch_timer: 0.0,
            output_power: 0.0,
            battery_level: 1.0, // Initialize battery to 100%
        }
    }
}

impl UVTorch {
    pub fn calculate_output_power(&self) -> f32 {
        if self.display_glitch_timer > 0.0 {
            return self.display_glitch_timer * 0.5; // Weaker flickering than flashlight
        }

        match self.enabled {
            false => 0.0,
            true => 2.0 * (self.battery_level.sqrt() + 0.05),
        }
    }

    pub fn update_output_power(&mut self) {
        let new_power = self.calculate_output_power();
        self.output_power = (self.output_power * 10.0 + new_power) / 11.0;
    }
}

impl GearUsable for UVTorch {
    fn get_sprite_idx(&self) -> GearSpriteID {
        if self.display_glitch_timer > 0.0 {
            // Flicker between on and off states during glitch
            if random_seed::rng().random_range(0.0..1.0) < 0.3 {
                return GearSpriteID::UVTorchOff;
            }
        }

        match self.enabled {
            true => GearSpriteID::UVTorchOn,
            false => GearSpriteID::UVTorchOff,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "UV Torch"
    }

    fn get_description(&self) -> &'static str {
        "Ultraviolet light that can be used to expose evidence invisible to the naked eye since some substances react to it and glow."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);

        // Show garbled text when glitching
        if self.enabled && self.display_glitch_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..4) {
                0 => "Bat---y: E--OR",
                1 => "UV Status: -.--%",
                2 => "WAVEL--GTH FA--T",
                _ => "CALIB---ING...",
            };
            return format!("{name}: {on_s}\n{garbled}");
        }

        let msg = if self.enabled {
            format!("Battery: {:>3.0}%", self.battery_level * 100.0)
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn update(&mut self, gs: &mut super::GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        // Reduce battery if enabled
        if self.enabled && self.display_glitch_timer <= 0.0 {
            self.battery_level -= 0.0001 * gs.time.delta_secs();
            if self.battery_level < 0.0 {
                self.battery_level = 0.0;
                self.enabled = false;
            }
        }

        // Decrement glitch timer if active
        if self.display_glitch_timer > 0.0 {
            self.display_glitch_timer -= gs.time.delta_secs();

            // Play static/interference sounds when glitching
            if self.enabled && random_seed::rng().random_range(0.0..1.0) < 0.2 {
                gs.play_audio("sounds/effects-chirp-short.ogg".into(), 0.3, pos);
            }
        }

        // Apply EMI if warning is active and we're electronic
        if let Some(ghost_pos) = &gs.bf.ghost_warning_position {
            let distance2 = pos.distance2(ghost_pos);
            self.apply_electromagnetic_interference(gs.bf.ghost_warning_intensity, distance2);
        }

        self.update_output_power();
    }

    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        if self.display_glitch_timer <= 0.0 {
            self.enabled = !self.enabled;
        }
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn power(&self) -> f32 {
        self.output_power
    }

    fn color(&self) -> Color {
        if self.display_glitch_timer > 0.0 {
            // Flicker to a different color when glitching
            let flicker = random_seed::rng().random_range(0.0..1.0);
            if flicker < 0.3 {
                return Color::srgb(0.3, 0.1, 0.5); // Dimmer purple
            }
        }
        // Normal color - Violet
        Color::srgb(0.60, 0.25, 1.00)
    }

    fn is_electronic(&self) -> bool {
        true
    }

    fn apply_electromagnetic_interference(&mut self, warning_level: f32, distance2: f32) {
        if warning_level < 0.0001 || !self.enabled {
            return;
        }
        let mut rng = random_seed::rng();

        // Scale effect by distance and warning level
        let effect_strength = warning_level * (100.0 / distance2).min(1.0);

        // Random UV glitches
        if rng.random_range(0.0..1.0) < effect_strength.powi(2) {
            // Turn off occasionally or glitch the display
            self.display_glitch_timer = rng.random_range(0.2..0.6);
        }
    }
}

impl From<UVTorch> for Gear {
    fn from(value: UVTorch) -> Self {
        Gear::new_from_kind(GearKind::UVTorch, value.box_clone())
    }
}
