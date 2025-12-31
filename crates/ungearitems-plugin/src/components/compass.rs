use bevy::prelude::*;
use unfoundation_core::types::gear::GearSpriteID;
use ungear_core::types::gear::utils::on_off;
use ungear_core::{GearSprite, ItemName, StatusText};
pub use ungearitems_core::components::compass::Compass;
use uninteraction_core::Toggleable;

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
