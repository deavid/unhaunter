use bevy::prelude::*;
use ungear_core::components::core::{Battery, Electronic, GearSprite, ItemName, StatusText};
use ungear_core::types::gear::sprite_id::GearSpriteID;
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::ionmeter::IonMeter;
use uninteraction_core::interaction::Toggleable;
use unmetrics_core::metrics::SendMetric;

use crate::metrics;

pub(crate) fn update_ionmeter(
    mut q_ionmeter: Query<
        (
            &mut StatusText,
            &mut GearSprite,
            &Toggleable,
            &mut Battery,
            &Electronic,
            &ItemName,
        ),
        With<IonMeter>,
    >,
) {
    let measure = metrics::IONMETER_UPDATE.time_measure();
    for (mut status, mut sprite, toggle, mut battery, electronic, name) in q_ionmeter.iter_mut() {
        // Update Battery Drain Rate
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };

        sprite.0 = if toggle.is_on {
            GearSpriteID::IonMeter0.to_visual_key()
        } else {
            GearSpriteID::IonMeterOff.to_visual_key()
        };

        let on_s = on_off(toggle.is_on);
        let msg = if toggle.is_on {
            if electronic.glitch_timer > 0.0 {
                "Reading: ERR".to_string()
            } else {
                "Reading: 32eV".to_string()
            }
        } else {
            "".to_string()
        };
        status.0 = format!("{}: {}\n{}", name.0, on_s, msg);
    }

    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_ionmeter);
}
