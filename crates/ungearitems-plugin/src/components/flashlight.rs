use unaudiospatial_core::emitter::AudioEmitter;
use uncommon_app_core::random_seed;
use ungear_core::components::core::{Battery, Electronic, GearSprite, ItemName, StatusText};
use uninteraction_core::interaction::{Toggleable, Triggered};
use unlight_core::components::LightEmitter;
use unmetrics_core::metrics::SendMetric;
use unspatial_core::position::Position;

use crate::metrics;

use bevy::prelude::*;
use enum_iterator::Sequence;
use rand::RngExt;
use ungear_core::types::gear::sprite_id::GearSpriteID;
pub(crate) use ungearitems_core::components::flashlight::{
    Flashlight, FlashlightSkin, FlashlightStatus,
};
use unreplicon_core::ownership::LocallyOwned;

pub(crate) fn update_flashlight_skeleton(
    mut commands: Commands,
    mut q_flashlight: Query<
        (
            Entity,
            &mut Flashlight,
            &mut Toggleable,
            &Battery,
            &Electronic,
            Option<&Triggered>,
        ),
        With<LocallyOwned>,
    >,
) {
    let measure = metrics::FLASHLIGHT_UPDATE.time_measure();
    for (entity, mut flashlight, mut toggle, battery, electronic, triggered) in
        q_flashlight.iter_mut()
    {
        // Handle Trigger
        if triggered.is_some() && electronic.glitch_timer <= 0.0 {
            let next_status = flashlight.status.next().unwrap_or_default();
            let is_battery_ok = battery.level > 0.0;
            if next_status == FlashlightStatus::Off || is_battery_ok {
                flashlight.status = next_status;
            } else if flashlight.status != FlashlightStatus::Off {
                flashlight.status = FlashlightStatus::Off;
            }
            // Remove Triggered now that we've processed it
            commands.entity(entity).remove::<Triggered>();
        }

        // Sync Toggleable with FlashlightStatus
        toggle.is_on = flashlight.status != FlashlightStatus::Off;
    }

    measure.end_ms();
}

pub(crate) fn update_flashlight_skin(
    mut q_flashlight: Query<(
        &mut Flashlight,
        &mut FlashlightSkin,
        &mut LightEmitter,
        &mut StatusText,
        &mut GearSprite,
        &Electronic,
        &mut Battery,
        &ItemName,
        &Position,
        Option<&LocallyOwned>,
    )>,
    mut ga: AudioEmitter,
) {
    let measure = metrics::FLASHLIGHT_UPDATE.time_measure();
    for (
        mut flashlight,
        mut skin,
        mut flashlight_render,
        mut status,
        mut sprite,
        electronic,
        mut battery,
        name,
        pos,
        locally_owned,
    ) in q_flashlight.iter_mut()
    {
        skin.frame_counter = skin.frame_counter.wrapping_add(1) % 210;

        if skin.frame_counter % 5 == 0 {
            skin.rand = random_seed::rng().random_range(0..12);
            const HS_MASS: f32 = 2.0;
            skin.heatsink_temp = (skin.heatsink_temp * HS_MASS + skin.inner_temp) / (HS_MASS + 1.0);
        }

        // Update Battery Drain Rate (only for local authority)
        if locally_owned.is_some() {
            battery.drain_rate = match flashlight.status {
                FlashlightStatus::Off => 0.0,
                FlashlightStatus::Low => 4.0,
                FlashlightStatus::Mid => 16.0,
                FlashlightStatus::High => 64.0,
            } / 5000.0;
        }

        if electronic.glitch_timer <= 0.0 {
            if battery.level <= 0.0 && locally_owned.is_some() {
                flashlight.status = FlashlightStatus::Off;
            }
            skin.inner_temp += skin.output_power / 50000.0;
            skin.inner_temp /= 1.00032;
            if skin.inner_temp > 1.0 && flashlight.status != FlashlightStatus::Off {
                if locally_owned.is_some() {
                    flashlight.status = FlashlightStatus::Off;
                }
                ga.play_audio("sounds/effects-dingdingding.ogg".into(), 0.7, pos);
            }
        }

        skin.update_output_power(&flashlight.status, battery.level, electronic.glitch_timer);

        // Sync with Render Component
        flashlight_render.power = skin.output_power;
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
        } else if skin.rand == 0 {
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
        let overheat = if skin.heatsink_temp > 0.8 {
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
            let heat_temp = 15.0 + skin.heatsink_temp * 70.0;
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

    measure.end_ms();
}

fn hydrate_flashlight_skin(
    mut commands: Commands,
    q_new: Query<Entity, (Added<Flashlight>, Without<FlashlightSkin>)>,
) {
    for entity in q_new.iter() {
        commands.entity(entity).insert(FlashlightSkin::default());
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        update_flashlight_skeleton
            .before(ungearitems_core::GearStateExportSet)
            .run_if(resource_exists::<unreplicon_core::resources::LocalPlayerRole>),
    );
    app.add_systems(
        Update,
        (hydrate_flashlight_skin, update_flashlight_skin)
            .run_if(resource_exists::<unreplicon_core::resources::LocalPlayerRole>),
    );
}
