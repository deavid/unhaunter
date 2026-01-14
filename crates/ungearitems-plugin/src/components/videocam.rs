use bevy::prelude::*;
use rand::Rng;
use unfoundation_core::random_seed;
use unfoundation_core::types::gear::{EquipmentPosition, GearSpriteID};
use ungear_core::components::core::{Battery, Electronic, GearSprite, StatusText};
use ungear_core::gear_stuff::GearStuff;
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::videocam::Videocam;
use uninteraction_core::interaction::Toggleable;
use unspatial_core::position::Position;

pub(crate) fn update_videocam(
    _gs: GearStuff,
    mut q_videocam: Query<(
        &mut Videocam,
        &mut StatusText,
        &mut GearSprite,
        &mut Toggleable,
        &mut Battery,
        &Electronic,
        &Position,
        &EquipmentPosition,
    )>,
) {
    for (_videocam, mut status, mut sprite, toggle, mut battery, electronic, _pos, _ep) in
        q_videocam.iter_mut()
    {
        let mut rng = random_seed::rng();

        // Update Battery Drain Rate
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };

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
        sprite.0 = GearSpriteID::Videocam;
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_videocam);
}
