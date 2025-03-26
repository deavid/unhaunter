use super::{Gear, GearKind, GearSpriteID, GearUsable, on_off};
use bevy::prelude::*;
use rand::Rng;
use uncore::components::board::position::Position;
use uncore::random_seed;
use uncore::types::gear::equipmentposition::EquipmentPosition;

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct Videocam {
    pub enabled: bool,
    pub display_glitch_timer: f32,
}

impl GearUsable for Videocam {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => GearSpriteID::Videocam,
            false => GearSpriteID::Videocam,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Video Camera NV"
    }

    fn get_description(&self) -> &'static str {
        "Mainly used for its infrared night vision, it can also transmit images to the van in real time."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);

        // Show garbled text when glitching
        if self.display_glitch_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..4) {
                0 => "Signal: --LOST--",
                1 => "Transmitting...FA--",
                2 => "NIGHT V---N F---",
                _ => "CAMERA OFFL---",
            };
            return format!("{name}: {on_s}\n{garbled}");
        }

        let msg = if self.enabled {
            "NIGHT VISION ON".to_string()
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        self.enabled = !self.enabled;
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn power(&self) -> f32 {
        match self.enabled {
            false => 0.0,
            true => 35.0,
        }
    }

    fn color(&self) -> Color {
        if self.display_glitch_timer > 0.0 {
            // Shift color towards red during glitch
            return Color::srgb(0.4, 0.5, 0.01);
        }
        // Green-Cyan (for NightVision)
        Color::srgb(0.01, 1.00, 0.70)
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

        // Random glitches
        if rng.random_range(0.0..1.0) < effect_strength.powi(2) {
            // Jumble numbers temporarily
            self.display_glitch_timer = rng.random_range(0.2..0.6);
        }
    }

    fn update(&mut self, gs: &mut super::GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        // Decrement glitch timer if active
        if self.display_glitch_timer > 0.0 {
            self.display_glitch_timer -= gs.time.delta_secs();
        }

        // Apply EMI if warning is active and we're electronic
        if let Some(ghost_pos) = &gs.bf.ghost_warning_position {
            let distance2 = pos.distance2(ghost_pos);
            self.apply_electromagnetic_interference(gs.bf.ghost_warning_intensity, distance2);
        }
    }
}

impl From<Videocam> for Gear {
    fn from(value: Videocam) -> Self {
        Gear::new_from_kind(GearKind::Videocam, value.box_clone())
    }
}
