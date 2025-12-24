use super::{GearSpriteID, on_off};
use bevy::prelude::*;
use uncore_components::{Battery, Electronic, GearSprite, ItemName, StatusText, Toggleable};

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct Photocam {}

pub fn update_photocam(
    mut q_photocam: Query<
        (
            &mut StatusText,
            &mut GearSprite,
            &Toggleable,
            &mut Battery,
            &Electronic,
            &ItemName,
        ),
        With<Photocam>,
    >,
) {
    for (mut status, mut sprite, toggle, mut battery, electronic, name) in q_photocam.iter_mut() {
        // Update Battery Drain Rate
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };

        sprite.0 = if toggle.is_on {
            GearSpriteID::PhotocamFlash2
        } else {
            GearSpriteID::Photocam
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
}

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, update_photocam);
}
