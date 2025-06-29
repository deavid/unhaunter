use bevy::prelude::*;
use uncore::{
    components::player_sprite::PlayerSprite,
    resources::{current_evidence_readings::CurrentEvidenceReadings, looking_gear::LookingGear},
    types::{evidence::Evidence, gear_kind::GearKind},
};
use ungear::{
    components::playergear::PlayerGear,
    types::gear::Gear, // The actual Gear struct
};

// This system is responsible for determining what evidence the player *perceives*
// from their handheld gear's UI and sound, and reporting that to CurrentEvidenceReadings.
// Environmental evidences (Orbs, UV Ecto, RL Pres) are handled by maplight.rs.
fn update_current_evidence_readings_from_player_perception_system(
    mut evidence_readings: ResMut<CurrentEvidenceReadings>,
    player_query: Query<(Entity, &PlayerGear, &PlayerSprite)>, // PlayerSprite for controls for LookingGear
    looking_gear: Res<LookingGear>,
    time: Res<Time>,
) {
    let Ok((_player_entity, player_gear, _player_sprite)) = player_query.single() else {
        return;
    };
    let current_game_time_secs = time.elapsed_secs_f64();
    let delta_time_secs = time.delta_secs();

    // Helper closure to process a single piece of gear
    let mut process_gear = |gear: &Gear,
                            is_status_text_prominently_visible: bool,
                            is_icon_prominently_visible: bool| {
        if gear.kind == GearKind::None {
            return;
        }
        let Some(gear_data) = gear.data.as_ref() else {
            return;
        };

        if let Ok(evidence_type) = Evidence::try_from(&gear.kind) {
            let mut clarity = 0.0f32;

            if is_status_text_prominently_visible {
                clarity = clarity.max(gear_data.is_status_text_showing_evidence());
            }
            // Important: Use else if for icon if status text already gives max clarity for the same visual aspect
            // However, icon might show different aspect of evidence or be a fallback.
            // For now, max() handles if both contribute independently or redundantly.
            if is_icon_prominently_visible {
                clarity = clarity.max(gear_data.is_icon_showing_evidence());
            }

            // For handheld gear, if its sound indicates evidence, it's perceived.
            // The GearUsable trait method should return 0.0 if not making evidential sound.
            clarity = clarity.max(gear_data.is_sound_showing_evidence());

            evidence_readings.report_clarity(
                evidence_type,
                clarity,
                current_game_time_secs,
                delta_time_secs,
            );
        }
    };

    // --- Right Hand ---
    // Status text and icon are considered prominently visible for the right hand.
    process_gear(
        &player_gear.right_hand,
        true, // Status text visible
        true, // Icon visible
    );

    // --- Left Hand ---
    let left_hand_status_text_visible =
        player_gear.left_hand.kind != GearKind::None && (looking_gear.held); // || keyboard_input.pressed(player_sprite.controls.left_hand_look));
    // LookingGear.held should be sufficient if updated correctly.
    let left_hand_icon_visible = player_gear.left_hand.kind != GearKind::None;
    process_gear(
        &player_gear.left_hand,
        left_hand_status_text_visible,
        left_hand_icon_visible,
    );

    // --- "Next" Inventory Slot (Icon for [Q] cycle) ---
    if let Some(next_gear_in_q_slot) = player_gear.get_next_non_empty() {
        if next_gear_in_q_slot.kind != GearKind::None {
            process_gear(
                &next_gear_in_q_slot,
                false, // "Next" item preview generally doesn't show full status text
                true,  // Icon is visible
            );
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        update_current_evidence_readings_from_player_perception_system,
    );
}
