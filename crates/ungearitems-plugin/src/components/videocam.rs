use bevy::prelude::*;
use rand::RngExt;
use unfoundation_core::random_seed;
use ungear_core::components::core::{Battery, Electronic, GearSprite, StatusText};
use ungear_core::types::gear::EquipmentPosition;
use ungear_core::types::gear::sprite_id::GearSpriteID;
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::videocam::Videocam;
use uninteraction_core::interaction::Toggleable;
use unmetrics_core::metrics::SendMetric;
use unrender_std::components::light::LightEmitter;
use unreplicon_core::ownership::LocallyOwned;

use crate::metrics;

pub(crate) fn update_videocam_skeleton(
    mut q_videocam: Query<(&mut Toggleable, &mut Battery), With<LocallyOwned>>,
) {
    let measure = metrics::VIDEOCAM_UPDATE.time_measure();
    for (toggle, mut battery) in q_videocam.iter_mut() {
        // Update Battery Drain Rate (only for local authority)
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };
    }

    measure.end_ms();
}

pub(crate) fn update_videocam_skin(
    mut q_videocam: Query<(
        &mut Videocam,
        &mut LightEmitter,
        &mut StatusText,
        &mut GearSprite,
        &Toggleable,
        &Electronic,
        &Battery,
        &EquipmentPosition,
    )>,
) {
    let measure = metrics::VIDEOCAM_UPDATE.time_measure();
    for (
        mut videocam,
        mut videocam_render,
        mut status,
        mut sprite,
        toggle,
        electronic,
        battery,
        _ep,
    ) in q_videocam.iter_mut()
    {
        // Update power (computed on all clients)
        let mut new_power = if toggle.is_on {
            35.0 * (battery.level.sqrt() + 0.1)
        } else {
            0.0
        };
        if toggle.is_on && electronic.glitch_timer > 0.0 {
            new_power = electronic.glitch_timer * 2.0;
        }
        videocam.output_power = (videocam.output_power * 8.0 + new_power) / 9.0;

        let mut rng = random_seed::rng();

        // Sync with Render Component
        videocam_render.power = videocam.output_power;
        if electronic.glitch_intensity > 0.01 {
            let base_color = Color::srgb(0.01, 1.00, 0.70);
            let mut color = base_color.to_srgba();
            let k = electronic.glitch_intensity.min(1.0);
            color.red += k * 0.7;
            color.blue -= k * 0.4;
            videocam_render.color = Color::Srgba(color);
        } else {
            videocam_render.color = Color::srgb(0.01, 1.00, 0.70);
        }

        // Update StatusText
        let name = "Video Camera NV";
        let on_s = on_off(toggle.is_on);

        // Show garbled text when glitching
        if toggle.is_on && electronic.glitch_timer > 0.0 {
            let garbled = match rng.random_range(0..4) {
                0 => "Signal: --LOST--",
                1 => "Transmitting...FA--",
                2 => "NIGHT V---N F---",
                _ => "CAMERA OFFL---",
            };
            status.0 = format!("{name}: {on_s}\n{garbled}");
        } else {
            let msg = if toggle.is_on {
                "NIGHT VISION ON".to_string()
            } else {
                "".to_string()
            };
            status.0 = format!("{name}: {on_s}\n{msg}");
        }

        // Update GearSprite
        sprite.0 = GearSpriteID::Videocam.to_visual_key();
    }

    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        update_videocam_skeleton.run_if(resource_exists::<untypes_core::roles::LocalPlayerRole>),
    );
    app.add_systems(
        Update,
        update_videocam_skin.run_if(resource_exists::<untypes_core::roles::LocalPlayerRole>),
    );
}
