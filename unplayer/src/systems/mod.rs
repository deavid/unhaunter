pub mod grabdrop;
pub mod hide;
pub mod keyboard;
pub mod mouse;
pub mod sanityhealth;

use bevy::prelude::App;

pub(crate) fn app_setup(app: &mut App) {
    grabdrop::app_setup(app);
    hide::app_setup(app);
    keyboard::app_setup(app);
    mouse::app_setup(app);
    sanityhealth::app_setup(app);
}
