use super::systems::grabdrop::{
    deploy_gear, drop_object, grab_object, retrieve_gear, update_held_object_position,
};
use super::systems::hide::{hide_player, unhide_player};
use super::systems::keyboard::keyboard_player;
use super::systems::sanityhealth::{lose_sanity, recover_sanity, visual_health};
use bevy::prelude::*;
use uncore::states::GameState;
use uncore::systems::animation::animate_sprite;

pub fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            keyboard_player,
            lose_sanity,
            visual_health,
            animate_sprite,
            update_held_object_position,
            deploy_gear,
            retrieve_gear,
            grab_object,
            drop_object,
            hide_player,
            unhide_player,
        )
            .run_if(in_state(GameState::None)),
    )
    .add_systems(Update, recover_sanity.run_if(in_state(GameState::Truck)));
}
