use super::{EquipmentPosition, Gear, GearKind, GearSpriteID, GearStuff, on_off};
use bevy::prelude::*;
use uncore_components::{Battery, Electronic, GearSprite, ItemName, StatusText, Toggleable};
use ungear::gear_usable::GearUsable;
use unspatial::Position;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct IonMeter {}

impl GearUsable for IonMeter {
    fn get_display_name(&self) -> &'static str {
        "Ion Meter"
    }

    fn get_description(&self) -> &'static str {
        "Measures ion concentration in the air."
    }

    fn get_status(&self) -> String {
        "".to_string()
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {}

    fn get_sprite_idx(&self) -> GearSpriteID {
        GearSpriteID::IonMeterOff
    }

    fn update(&mut self, _gs: &mut GearStuff, _pos: &Position, _ep: &EquipmentPosition) {}

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<IonMeter> for Gear {
    fn from(value: IonMeter) -> Self {
        Gear::new_from_kind(GearKind::IonMeter, value.box_clone())
    }
}

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
