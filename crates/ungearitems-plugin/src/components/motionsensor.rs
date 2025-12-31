use bevy::prelude::*;
use uncore_foundation::types::gear::GearSpriteID;
use ungear_core::types::gear::utils::on_off;
use ungear_core::{Battery, Electronic, GearSprite, ItemName, StatusText};
pub use ungearitems_core::components::motionsensor::MotionSensor;
use uninteraction_core::Toggleable;

pub fn update_motionsensor(
    mut q_motionsensor: Query<
        (
            &mut StatusText,
            &mut GearSprite,
            &Toggleable,
            &mut Battery,
            &Electronic,
            &ItemName,
        ),
        With<MotionSensor>,
    >,
) {
    for (mut status, mut sprite, toggle, mut battery, electronic, name) in q_motionsensor.iter_mut()
    {
        // Update Battery Drain Rate
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };

        sprite.0 = GearSpriteID::MotionSensor;

        let on_s = on_off(toggle.is_on);
        let msg = if toggle.is_on {
            if electronic.glitch_timer > 0.0 {
                "ERR".to_string()
            } else {
                "--".to_string()
            }
        } else {
            "".to_string()
        };
        status.0 = format!("{}: {}\n{}", name.0, on_s, msg);
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, update_motionsensor);
}
