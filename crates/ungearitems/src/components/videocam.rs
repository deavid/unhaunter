use super::{EquipmentPosition, Gear, GearKind, GearSpriteID, GearUsable, on_off};
use bevy::prelude::*;
use rand::Rng;
use uncore_components::{Battery, Electronic, GearSprite, StatusText, Toggleable};
use uncore_foundation::random_seed;
use ungear::gear_stuff::GearStuff;
use unspatial::Position;

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct Videocam {}

pub fn update_videocam(
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

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, update_videocam);
}

impl GearUsable for Videocam {
    fn get_display_name(&self) -> &'static str {
        "Video Camera NV"
    }

    fn get_description(&self) -> &'static str {
        "Mainly used for its infrared night vision, it can also transmit images to the van in real time."
    }

    fn get_status(&self) -> String {
        "".to_string()
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {}

    fn get_sprite_idx(&self) -> GearSpriteID {
        GearSpriteID::Videocam
    }

    fn update(&mut self, _gs: &mut GearStuff, _pos: &Position, _ep: &EquipmentPosition) {}

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<Videocam> for Gear {
    fn from(value: Videocam) -> Self {
        Gear::new_from_kind(GearKind::Videocam, value.box_clone())
    }
}
