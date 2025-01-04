pub mod game;
mod mainmenu;
mod maplight;
mod npchelp;

use bevy::prelude::App;

pub fn app_setup(app: &mut App) {
    mainmenu::app_setup(app);
    maplight::app_setup(app);
    npchelp::app_setup(app);
}
