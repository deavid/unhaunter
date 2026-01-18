use bevy::prelude::*;
use rand::Rng;
use unfoundation_core::random_seed;
use ungear_core::components::core::{Battery, Electronic, GearSprite, ItemName, StatusText};
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::uvtorch::UVTorch;
use uninteraction_core::interaction::Toggleable;
use unrender_std::resources::sprite_registry::GearSpriteID;
use unsound_core::emitter::SoundEmitter;
use unspatial_core::position::Position;

pub(crate) trait UVTorchExt {
    fn calculate_output_power(&self, battery_level: f32, glitch_timer: f32) -> f32;
    fn update_output_power(&mut self, battery_level: f32, glitch_timer: f32);
}

impl UVTorchExt for UVTorch {
    fn calculate_output_power(&self, battery_level: f32, glitch_timer: f32) -> f32 {
        if glitch_timer > 0.0 {
            return glitch_timer * 0.5; // Weaker flickering than flashlight
        }

        match self.enabled {
            false => 0.0,
            true => 2.0 * (battery_level.sqrt() + 0.05),
        }
    }

    fn update_output_power(&mut self, battery_level: f32, glitch_timer: f32) {
        let new_power = self.calculate_output_power(battery_level, glitch_timer);
        self.output_power = (self.output_power * 10.0 + new_power) / 11.0;
    }
}

pub(crate) fn update_uvtorch(
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
    mut ga: SoundEmitter,
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
            ga.play_audio("sounds/effects-chirp-short.ogg".into(), 0.3, pos);
        }

        uvtorch.update_output_power(battery.level, electronic.glitch_timer);

        // Update Sprite
        sprite.0 = if electronic.glitch_timer > 0.0 {
            if random_seed::rng().random_range(0.0..1.0) < 0.3 {
                GearSpriteID::UVTorchOff.to_visual_key()
            } else {
                GearSpriteID::UVTorchOn.to_visual_key()
            }
        } else if uvtorch.enabled {
            GearSpriteID::UVTorchOn.to_visual_key()
        } else {
            GearSpriteID::UVTorchOff.to_visual_key()
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

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_uvtorch);
}
