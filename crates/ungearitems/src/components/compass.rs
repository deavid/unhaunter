use super::{EquipmentPosition, Gear, GearKind, GearSpriteID, GearStuff, on_off};
use bevy::prelude::*;
use uncore_components::{GearSprite, ItemName, StatusText, Toggleable};
use ungear::gear_usable::GearUsable;
use unspatial::Position;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct Compass {}

impl GearUsable for Compass {
    fn get_display_name(&self) -> &'static str {
        "Compass"
    }

    fn get_description(&self) -> &'static str {
        "A simple compass to help you find your way."
    }

    fn get_status(&self) -> String {
        "".to_string()
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {}

    fn get_sprite_idx(&self) -> GearSpriteID {
        GearSpriteID::Compass
    }

    fn update(&mut self, _gs: &mut GearStuff, _pos: &Position, _ep: &EquipmentPosition) {}

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<Compass> for Gear {
    fn from(value: Compass) -> Self {
        Gear::new_from_kind(GearKind::Compass, value.box_clone())
    }
}

pub fn update_compass(
    mut q_compass: Query<(&mut StatusText, &mut GearSprite, &Toggleable, &ItemName), With<Compass>>,
) {
    for (mut status, mut sprite, toggle, name) in q_compass.iter_mut() {
        sprite.0 = GearSpriteID::Compass;

        let on_s = on_off(toggle.is_on);
        let msg = if toggle.is_on {
            "You managed to 'turn on' a compass".to_string()
        } else {
            "".to_string()
        };
        status.0 = format!("{}: {}\n{}", name.0, on_s, msg);
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, update_compass);
}
