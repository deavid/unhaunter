use unfoundation_core::random_seed;
use ungear_core::components::core::{Battery, Electronic, GearSprite, ItemName, StatusText};
use uninteraction_core::interaction::{Toggleable, Triggered};
use unrender_std::components::light::LightEmitter;
use unsound_core::emitter::SoundEmitter;
use unspatial_core::position::Position;

use bevy::prelude::*;
use enum_iterator::Sequence;
use rand::Rng;
pub(crate) use ungearitems_core::components::flashlight::{Flashlight, FlashlightStatus};
use unrender_std::resources::sprite_registry::GearSpriteID;
use untypes_core::cli::{CliOptions, is_host};

pub(crate) fn update_flashlight(
    mut commands: Commands,
    mut q_flashlight: Query<(
        Entity,
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
    mut ga: SoundEmitter,
    cli: Res<CliOptions>,
) {
    let is_host = is_host(cli);
    for (
        entity,
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
            // Remove Triggered now that we've processed it
            commands.entity(entity).remove::<Triggered>();
        }

        // Sync Toggleable with FlashlightStatus
        if is_host {
            toggle.is_on = flashlight.status != FlashlightStatus::Off;
        } else if !toggle.is_on && flashlight.status != FlashlightStatus::Off {
            // If the host says it's off, it's off.
            flashlight.status = FlashlightStatus::Off;
        }

        // Update Logic
        flashlight.frame_counter += 1;
        flashlight.frame_counter %= 210;
        if is_host {
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
                    ga.play_audio("sounds/effects-dingdingding.ogg".into(), 0.7, pos);
                }
            }
        }

        flashlight.update_output_power(battery.level, electronic.glitch_timer);

        // Sync with Render Component
        flashlight_render.power = flashlight.output_power;
        if electronic.glitch_intensity > 0.01 {
            let mut color = Color::WHITE.to_srgba();
            let k = electronic.glitch_intensity.min(1.0);
            color.red = 1.0;
            color.green = 1.0 - k * 0.5;
            color.blue = 1.0 - k * 0.7;
            flashlight_render.color = Color::Srgba(color);
        } else {
            flashlight_render.color = Color::WHITE;
        }

        // Update Sprite
        sprite.0 = if electronic.glitch_timer > 0.0 {
            GearSpriteID::Flashlight3.to_visual_key()
        } else if flashlight.rand == 0 {
            match flashlight.status {
                FlashlightStatus::Off => GearSpriteID::FlashlightOff.to_visual_key(),
                FlashlightStatus::Low => GearSpriteID::Flashlight2.to_visual_key(),
                FlashlightStatus::Mid => GearSpriteID::Flashlight1.to_visual_key(),
                FlashlightStatus::High => GearSpriteID::Flashlight2.to_visual_key(),
            }
        } else {
            match flashlight.status {
                FlashlightStatus::Off => GearSpriteID::FlashlightOff.to_visual_key(),
                FlashlightStatus::Low => GearSpriteID::Flashlight1.to_visual_key(),
                FlashlightStatus::Mid => GearSpriteID::Flashlight2.to_visual_key(),
                FlashlightStatus::High => GearSpriteID::Flashlight3.to_visual_key(),
            }
        };

        // Update Status Text
        let on_s = flashlight.status.as_ref();
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

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_flashlight);
}
