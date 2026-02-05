use bevy::prelude::*;
use ungear_core::components::core::{Battery, Electronic, GearSprite, ItemName, StatusText};
use ungear_core::types::gear::sprite_id::GearSpriteID;
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::thermalimager::ThermalImager;
use uninteraction_core::interaction::Toggleable;
use unmetrics_core::metrics::SendMetric;

use crate::metrics;

pub(crate) fn update_thermalimager(
    mut q_thermalimager: Query<
        (
            &mut StatusText,
            &mut GearSprite,
            &Toggleable,
            &mut Battery,
            &Electronic,
            &ItemName,
        ),
        With<ThermalImager>,
    >,
) {
    let measure = metrics::THERMALIMAGER_UPDATE.time_measure();
    for (mut status, mut sprite, toggle, mut battery, electronic, name) in
        q_thermalimager.iter_mut()
    {
        // Update Battery Drain Rate
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };

        sprite.0 = if toggle.is_on {
            GearSpriteID::ThermalImagerOn.to_visual_key()
        } else {
            GearSpriteID::ThermalImagerOff.to_visual_key()
        };

        let on_s = on_off(toggle.is_on);
        let msg = if toggle.is_on {
            if electronic.glitch_timer > 0.0 {
                "Battery: ERR".to_string()
            } else {
                format!("Battery: {:>3.0}%", battery.level * 100.0)
            }
        } else {
            "".to_string()
        };
        status.0 = format!("{}: {}\n{}", name.0, on_s, msg);
    }

    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_thermalimager);
}
