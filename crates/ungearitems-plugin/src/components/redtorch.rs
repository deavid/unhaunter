use bevy::prelude::*;
use rand::Rng;
use unfoundation_core::random_seed;
use ungear_core::components::core::{Battery, Electronic, GearSprite, ItemName, StatusText};
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::redtorch::RedTorch;
use uninteraction_core::interaction::Toggleable;
use unrender_std::components::light::LightEmitter;
use unrender_std::resources::sprite_registry::GearSpriteID;
use unsound_core::emitter::SoundEmitter;
use unspatial_core::position::Position;

pub(crate) fn update_redtorch(
    mut q_redtorch: Query<(
        &mut RedTorch,
        &mut LightEmitter,
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
    for (
        mut redtorch,
        mut redtorch_render,
        mut status,
        mut sprite,
        toggle,
        mut battery,
        electronic,
        pos,
        name,
    ) in q_redtorch.iter_mut()
    {
        // Sync internal enabled with Toggleable
        redtorch.enabled = toggle.is_on;

        // Update Battery Drain Rate
        battery.drain_rate = if redtorch.enabled { 0.0001 } else { 0.0 };

        // Play static/interference sounds when glitching
        if electronic.glitch_timer > 0.0
            && redtorch.enabled
            && random_seed::rng().random_range(0.0..1.0) < 0.2
        {
            ga.play_audio("sounds/effects-chirp-short.ogg".into(), 0.3, pos);
        }

        // Update power
        let mut new_power = if redtorch.enabled {
            2.5 * (battery.level.sqrt() + 0.1)
        } else {
            0.0
        };
        if redtorch.enabled && electronic.glitch_timer > 0.0 {
            new_power = electronic.glitch_timer * 0.3;
        }
        redtorch.output_power = (redtorch.output_power * 5.0 + new_power) / 6.0;

        // Sync with Render Component
        redtorch_render.power = redtorch.output_power;
        if electronic.glitch_intensity > 0.01 {
            let base_color = Color::srgb(1.0, 0.20, 0.07);
            let mut color = base_color.to_srgba();
            let k = electronic.glitch_intensity.min(1.0);
            color.green += k * 0.6;
            color.blue += k * 0.3;
            redtorch_render.color = Color::Srgba(color);
        } else {
            redtorch_render.color = Color::srgb(1.0, 0.20, 0.07);
        }

        // Update Sprite
        sprite.0 = if electronic.glitch_timer > 0.0 {
            if random_seed::rng().random_range(0.0..1.0) < 0.3 {
                GearSpriteID::RedTorchOff.to_visual_key()
            } else {
                GearSpriteID::RedTorchOn.to_visual_key()
            }
        } else if redtorch.enabled {
            GearSpriteID::RedTorchOn.to_visual_key()
        } else {
            GearSpriteID::RedTorchOff.to_visual_key()
        };

        // Update Status Text
        let on_s = on_off(redtorch.enabled);
        if redtorch.enabled && electronic.glitch_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..4) {
                0 => "Bat---y: E--OR",
                1 => "UV Status: -.--%",
                2 => "WAVEL--GTH FA--T",
                _ => "CALIB---ING...",
            };
            status.0 = format!("{}: {}\n{}", name.0, on_s, garbled);
        } else {
            let msg = if redtorch.enabled {
                format!("Battery: {:>3.0}%", battery.level * 100.0)
            } else {
                "".to_string()
            };
            status.0 = format!("{}: {}\n{}", name.0, on_s, msg);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_redtorch);
}
