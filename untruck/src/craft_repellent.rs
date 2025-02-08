use bevy::prelude::*;
use uncore::types::{gear_kind::GearKind, ghost::types::GhostType};
use ungear::components::playergear::{Hand, PlayerGear};
use ungearitems::prelude::RepellentFlask;

pub fn craft_repellent(playergear: &mut PlayerGear, ghost_type: GhostType) {
    // Check if the repellent exists in inventory, if not, create it:
    let flask_exists = playergear
        .as_vec()
        .iter()
        .any(|x| matches!(x.0.kind, GearKind::RepellentFlask));
    if !flask_exists {
        let old_rh = playergear.take_hand(&Hand::Right);
        playergear.right_hand = RepellentFlask::default().into();
        playergear.append(old_rh);
    }

    // Assume that one always exists. Retrieve the &mut reference:
    let mut inventory = playergear.as_vec_mut();
    let Some(flask) = inventory
        .iter_mut()
        .find(|x| matches!(x.0.kind, GearKind::RepellentFlask))
    else {
        error!("Flask not found??");
        return;
    };
    flask.0.data.as_mut().unwrap().do_fill_liquid(ghost_type);
}
