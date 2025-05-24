use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::{
    components::player_sprite::PlayerSprite, resources::looking_gear::LookingGear,
    types::evidence::Evidence,
};
use ungear::components::playergear::PlayerGear;
use unprofile::data::PlayerProfileData;

pub fn acknowledge_blinking_gear_hint_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<(&PlayerSprite, &PlayerGear)>,
    mut profile_data: ResMut<Persistent<PlayerProfileData>>,
    looking_gear: Res<LookingGear>,
) {
    for (player_sprite, player_gear) in player_query.iter() {
        let controls = &player_sprite.controls;

        if keyboard_input.just_pressed(controls.change_evidence) {
            let active_gear = match looking_gear.hand() {
                uncore::types::gear::equipmentposition::Hand::Left => &player_gear.left_hand,
                uncore::types::gear::equipmentposition::Hand::Right => &player_gear.right_hand,
            };

            if let Some(gear_data) = &active_gear.data {
                if gear_data.is_blinking_hint_active() {
                    if let Ok(evidence_type) = Evidence::try_from(&active_gear.kind) {
                        const HINT_ACKNOWLEDGE_THRESHOLD: u32 = 3;
                        let count = profile_data
                            .times_evidence_acknowledged_on_gear
                            .entry(evidence_type)
                            .or_insert(0);

                        if *count < HINT_ACKNOWLEDGE_THRESHOLD {
                            *count += 1;
                            info!(
                                "Acknowledged gear hint for {:?}. New count: {}",
                                evidence_type, *count
                            );
                            profile_data.set_changed();
                        }
                    }
                }
            }
        }
    }
}
