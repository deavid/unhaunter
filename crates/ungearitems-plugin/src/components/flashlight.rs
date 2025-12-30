use uncore_components::{
    Battery, Electronic, GearSprite, ItemName, LightEmitter, StatusText, Toggleable, Triggered,
};
use uncore_foundation::random_seed;
use ungear_core::gear_stuff::GearStuff;
use unspatial_core::Position;

use bevy::prelude::*;
use enum_iterator::Sequence;
use rand::Rng;
use uncore_foundation::types::gear::GearSpriteID;
pub use ungearitems_core::components::flashlight::{Flashlight, FlashlightStatus};

pub fn update_flashlight(
    mut q_flashlight: Query<(
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
    mut gs: GearStuff,
) {
    for (
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
        }

        // Sync Toggleable with FlashlightStatus
        toggle.is_on = flashlight.status != FlashlightStatus::Off;

        // Update Logic
        flashlight.frame_counter += 1;
        flashlight.frame_counter %= 210;
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
                gs.play_audio("sounds/effects-dingdingding.ogg".into(), 0.7, pos);
            }
        } else if flashlight.status != FlashlightStatus::Off {
            // If it was on and it's glitching, turn it off temporarily
            flashlight.status = FlashlightStatus::Off;
        } else if electronic.glitch_timer < 0.01 {
            // If it was off due to glitching and glitch is ending, turn it back on
            flashlight.status = FlashlightStatus::Low;
        }

        flashlight.update_output_power(battery.level, electronic.glitch_timer);

        // Sync with Render Component
        flashlight_render.power = flashlight.output_power;

        // Update Sprite
        sprite.0 = if electronic.glitch_timer > 0.0 {
            GearSpriteID::Flashlight3
        } else if flashlight.rand == 0 {
            match flashlight.status {
                FlashlightStatus::Off => GearSpriteID::FlashlightOff,
                FlashlightStatus::Low => GearSpriteID::Flashlight2,
                FlashlightStatus::Mid => GearSpriteID::Flashlight1,
                FlashlightStatus::High => GearSpriteID::Flashlight2,
            }
        } else {
            match flashlight.status {
                FlashlightStatus::Off => GearSpriteID::FlashlightOff,
                FlashlightStatus::Low => GearSpriteID::Flashlight1,
                FlashlightStatus::Mid => GearSpriteID::Flashlight2,
                FlashlightStatus::High => GearSpriteID::Flashlight3,
            }
        };

        // Update Status Text
        let on_s = flashlight.status.string();
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

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, update_flashlight);
}
