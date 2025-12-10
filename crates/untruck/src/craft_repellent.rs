use uncore::types::{gear::equipmentposition::Hand, gear_kind::GearKind, ghost::types::GhostType};
use ungear::components::playergear::PlayerGear;
use ungearitems::prelude::RepellentFlask;

/// Crafts a repellent for the specified ghost type.
/// Returns true if a new bottle was consumed (should count as a craft).
pub fn craft_repellent(playergear: &mut PlayerGear, ghost_type: GhostType) -> bool {
    // 1) Find or create a repellent flask
    if !playergear
        .as_vec()
        .iter()
        .any(|x| matches!(x.0.kind, GearKind::RepellentFlask))
    {
        let old_rh = playergear.take_hand(&Hand::Right);
        playergear.right_hand = RepellentFlask::default().into();
        playergear.append(old_rh);
    }

    // 2) Call do_fill_liquid and return the result
    playergear
        .as_vec_mut()
        .iter_mut()
        .find(|x| matches!(x.0.kind, GearKind::RepellentFlask))
        .unwrap()
        .0
        .data
        .as_mut()
        .unwrap()
        .do_fill_liquid(ghost_type)
}
