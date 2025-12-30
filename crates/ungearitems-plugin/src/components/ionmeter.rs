use bevy::prelude::*;
use uncore_components::{Battery, Electronic, GearSprite, ItemName, StatusText, Toggleable};
use uncore_foundation::types::gear::GearSpriteID;
use ungear::types::gear::utils::on_off;
pub use ungearitems_core::components::ionmeter::IonMeter;

pub fn update_ionmeter(
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
    for (mut status, mut sprite, toggle, mut battery, electronic, name) in q_ionmeter.iter_mut() {
        // Update Battery Drain Rate
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };

        sprite.0 = if toggle.is_on {
            GearSpriteID::IonMeter0
        } else {
            GearSpriteID::IonMeterOff
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
}

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, update_ionmeter);
}
