use bevy::prelude::*;
use rand::RngExt;
use unaudiospatial_core::emitter::AudioEmitter;
use uncommon_app_core::random_seed;
use ungear_core::components::core::{Battery, Electronic, GearSprite, ItemName, StatusText};
use ungear_core::types::gear::sprite_id::GearSpriteID;
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::uvtorch::{UVTorch, UVTorchSkin};
use uninteraction_core::interaction::Toggleable;
use unlight_core::components::LightEmitter;
use unmetrics_core::metrics::SendMetric;
use unreplicon_core::ownership::LocallyOwned;
use unspatial_core::position::Position;

use crate::metrics;

pub(crate) trait UVTorchSkinExt {
    fn calculate_output_power(enabled: bool, battery_level: f32, glitch_timer: f32) -> f32;
    fn update_output_power(&mut self, enabled: bool, battery_level: f32, glitch_timer: f32);
}

impl UVTorchSkinExt for UVTorchSkin {
    fn calculate_output_power(enabled: bool, battery_level: f32, glitch_timer: f32) -> f32 {
        if glitch_timer > 0.0 {
            return glitch_timer * 0.5; // Weaker flickering than flashlight
        }

        if !enabled {
            return 0.0;
        }

        4.0 * (battery_level.sqrt() + 0.05)
    }

    fn update_output_power(&mut self, enabled: bool, battery_level: f32, glitch_timer: f32) {
        let new_power = Self::calculate_output_power(enabled, battery_level, glitch_timer);
        self.output_power = (self.output_power * 10.0 + new_power) / 11.0;
    }
}

pub(crate) fn update_uvtorch_skeleton(
    mut q_uvtorch: Query<
        (
            &mut UVTorch,
            &mut Battery,
            &Toggleable,
            &Electronic,
            &Position,
        ),
        With<LocallyOwned>,
    >,
    mut ga: AudioEmitter,
) {
    let measure = metrics::UVTORCH_UPDATE.time_measure();
    for (mut uvtorch, mut battery, toggle, electronic, pos) in q_uvtorch.iter_mut() {
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
    }

    measure.end_ms();
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
        update_uvtorch_skeleton
            .before(ungearitems_core::GearStateExportSet)
            .run_if(resource_exists::<uncommon_app_core::roles::LocalPlayerRole>),
    );
    app.add_systems(
        Update,
        (hydrate_uvtorch_skin, update_uvtorch_skin)
            .run_if(resource_exists::<uncommon_app_core::roles::LocalPlayerRole>),
    );
}
