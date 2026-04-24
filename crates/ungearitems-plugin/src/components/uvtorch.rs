use bevy::prelude::*;
use rand::RngExt;
use uncommon_app_core::random_seed;
use ungear_core::components::core::{Battery, Electronic, GearSprite, ItemName, StatusText};
use ungear_core::types::gear::sprite_id::GearSpriteID;
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::uvtorch::{UVTorch, UVTorchSkin};
use unlight_core::components::LightEmitter;
use unmetrics_core::metrics::SendMetric;

use crate::metrics;

pub(crate) trait UVTorchSkinExt {
    fn calculate_output_power(enabled: bool, battery_level: f32, glitch_timer: f32) -> f32;
    fn update_output_power(&mut self, enabled: bool, battery_level: f32, glitch_timer: f32);
}

impl UVTorchSkinExt for UVTorchSkin {
    fn calculate_output_power(enabled: bool, battery_level: f32, glitch_timer: f32) -> f32 {
        if !enabled {
            return 0.0;
        }

        let normal_power = 4.0 * (battery_level.sqrt() + 0.05);
        if glitch_timer > 0.0 {
            // Keep EMI subtle: never dim below 50% of normal UV output.
            return (glitch_timer * 0.5).max(normal_power * 0.5);
        }

        normal_power
    }

    fn update_output_power(&mut self, enabled: bool, battery_level: f32, glitch_timer: f32) {
        let new_power = Self::calculate_output_power(enabled, battery_level, glitch_timer);
        self.output_power = (self.output_power * 10.0 + new_power) / 11.0;
    }
}

pub(crate) fn update_uvtorch_skin(
    mut q_uvtorch: Query<(
        &UVTorch,
        &mut UVTorchSkin,
        &mut LightEmitter,
        &mut StatusText,
        &mut GearSprite,
        &Electronic,
        &Battery,
        &ItemName,
    )>,
) {
    let measure = metrics::UVTORCH_UPDATE.time_measure();
    for (
        uvtorch,
        mut skin,
        mut uvtorch_render,
        mut status,
        mut sprite,
        electronic,
        battery,
        name,
    ) in q_uvtorch.iter_mut()
    {
        // Update skin state from skeleton
        skin.update_output_power(uvtorch.enabled, battery.level, electronic.glitch_timer);

        // Sync with Render Component
        uvtorch_render.power = skin.output_power;
        if electronic.glitch_intensity > 0.01 {
            let base_color = Color::srgb(0.60, 0.25, 1.00);
            let mut color = base_color.to_srgba();
            let k = electronic.glitch_intensity.min(1.0);
            color.red += k * 0.4;
            color.green += k * 0.4;
            uvtorch_render.color = Color::Srgba(color);
        } else {
            uvtorch_render.color = Color::srgb(0.40, 0.01, 1.00);
        }

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

    measure.end_ms();
}

fn hydrate_uvtorch_skin(
    mut commands: Commands,
    q_new: Query<Entity, (Added<UVTorch>, Without<UVTorchSkin>)>,
) {
    for entity in q_new.iter() {
        commands.entity(entity).insert(UVTorchSkin::default());
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (hydrate_uvtorch_skin, update_uvtorch_skin)
            .run_if(resource_exists::<unreplicon_core::resources::LocalPlayerRole>),
    );
}
