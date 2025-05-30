use super::{Gear, GearKind, GearSpriteID, GearUsable, on_off};
use bevy::prelude::*;
use rand::Rng;
use uncore::components::board::position::Position;
use uncore::random_seed;
use uncore::types::gear::equipmentposition::EquipmentPosition;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct RedTorch {
    pub enabled: bool,
    pub display_glitch_timer: f32,
    pub battery_level: f32,
}

impl Default for RedTorch {
    fn default() -> Self {
        Self {
            enabled: false,
            display_glitch_timer: 0.0,
            battery_level: 1.0, // Initialize battery to 100%
        }
    }
}

impl GearUsable for RedTorch {
    fn get_sprite_idx(&self) -> GearSpriteID {
        if self.display_glitch_timer > 0.0 {
            // Flicker between on and off states during glitch
            if random_seed::rng().random_range(0.0..1.0) < 0.3 {
                return GearSpriteID::RedTorchOff;
            }
        }
        match self.enabled {
            true => GearSpriteID::RedTorchOn,
            false => GearSpriteID::RedTorchOff,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Red Torch"
    }

    fn get_description(&self) -> &'static str {
        "A simple red light used by astronomers to see on the dark without losing night vision eye's adaptation. But the ghost might also react to this too."
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

    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        if self.can_enable() {
            self.enabled = !self.enabled;
        } else if self.is_enabled() {
            self.enabled = false;
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled && self.battery_level > 0.0 && self.display_glitch_timer <= 0.01
    }

    fn can_enable(&self) -> bool {
        self.battery_level > 0.0 && self.display_glitch_timer <= 0.01
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn power(&self) -> f32 {
        match self.enabled {
            false => 0.0,
            true => 2.5,
        }
    }

    fn color(&self) -> Color {
        if self.display_glitch_timer > 0.0 {
            // Flicker to a different color when glitching
            let flicker = random_seed::rng().random_range(0.0..1.0);
            if flicker < 0.3 {
                return Color::srgb(1.0, 0.40, 0.07); // A bit more orange
            }
        }
        // Red
        Color::srgb(1.0, 0.20, 0.07)
    }

    fn is_electronic(&self) -> bool {
        true
    }

    fn needs_darkness(&self) -> bool {
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
    }
}

impl From<RedTorch> for Gear {
    fn from(value: RedTorch) -> Self {
        Gear::new_from_kind(GearKind::RedTorch, value.box_clone())
    }
}
