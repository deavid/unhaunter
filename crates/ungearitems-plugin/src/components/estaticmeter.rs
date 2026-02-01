use bevy::prelude::*;
use ungear_core::components::core::{Battery, Electronic, GearSprite, ItemName, StatusText};
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::estaticmeter::EStaticMeter;
use uninteraction_core::interaction::Toggleable;
use ungear_core::types::gear::sprite_id::GearSpriteID;

pub(crate) fn update_estaticmeter(
    mut q_estaticmeter: Query<
        (
            &mut StatusText,
            &mut GearSprite,
            &Toggleable,
            &mut Battery,
            &Electronic,
            &ItemName,
        ),
        With<EStaticMeter>,
    >,
) {
    for (mut status, mut sprite, toggle, mut battery, electronic, name) in q_estaticmeter.iter_mut()
    {
        // Update Battery Drain Rate
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };

        sprite.0 = GearSpriteID::EStaticMeter.to_visual_key();

        let on_s = on_off(toggle.is_on);
        let msg = if toggle.is_on {
            if electronic.glitch_timer > 0.0 {
                "Reading: ERR".to_string()
            } else {
                "Reading: 200V/m".to_string()
            }
        } else {
            "".to_string()
        };
        status.0 = format!("{}: {}\n{}", name.0, on_s, msg);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_estaticmeter);
}
