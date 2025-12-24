use super::{EquipmentPosition, Gear, GearKind, GearSpriteID, GearStuff, on_off};
use bevy::prelude::*;
use uncore_components::{Battery, Electronic, GearSprite, ItemName, StatusText, Toggleable};
use ungear::gear_usable::GearUsable;
use unspatial::Position;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct Photocam {}

impl GearUsable for Photocam {
    fn get_display_name(&self) -> &'static str {
        "Photo Camera"
    }

    fn get_description(&self) -> &'static str {
        "Takes photos of paranormal phenomena."
    }

    fn get_status(&self) -> String {
        "".to_string()
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {}

    fn get_sprite_idx(&self) -> GearSpriteID {
        GearSpriteID::Photocam
    }

    fn update(&mut self, _gs: &mut GearStuff, _pos: &Position, _ep: &EquipmentPosition) {}

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<Photocam> for Gear {
    fn from(value: Photocam) -> Self {
        Gear::new_from_kind(GearKind::Photocam, value.box_clone())
    }
}

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
