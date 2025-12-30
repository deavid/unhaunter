use bevy::prelude::*;
use uncore_board::resources::board_data::BoardData;
use uncore_components::{EvidenceSensor, Toggleable};
use uncore_foundation::types::evidence::Evidence;
use ungear::components::playergear::PlayerGear;
use unghost_core::resources::current_evidence_readings::CurrentEvidenceReadings;
use unghost_core::resources::haunt_state::HauntState;
use unspatial_core::Position;
use untags_core::PlayerTag;

fn update_current_evidence_readings_from_player_perception_system(
    mut evidence_readings: ResMut<CurrentEvidenceReadings>,
    player_query: Query<(&PlayerGear, &Position), With<PlayerTag>>,
    q_evidence_sensor: Query<(&EvidenceSensor, &Toggleable)>,
    board_data: Res<BoardData>,
    haunt_state: Res<HauntState>,
    time: Res<Time>,
) {
    let Ok((player_gear, player_pos)) = player_query.single() else {
        return;
    };

    let delta_time = time.delta_secs();
    let current_time = time.elapsed_secs_f64();

    // We check both hands
    let hands = [player_gear.left_hand, player_gear.right_hand];

    for hand_entity in hands.into_iter().flatten() {
        if let Ok((sensor, toggle)) = q_evidence_sensor.get(hand_entity) {
            if !toggle.is_on {
                continue;
            }

            let evidence = sensor.evidence;
            let mut clarity = 0.0;

            match evidence {
                Evidence::FreezingTemp => {
                    // Check temperature at player position
                    let bpos = player_pos.to_board_position();
                    if let Some(&temp) = board_data.temperature_field.get(bpos.ndidx()) {
                        // If temp < 0.0, clarity is 1.0. If temp > 5.0, clarity is 0.0.
                        clarity = ((5.0 - temp) / 5.0).clamp(0.0, 1.0);
                    }
                }
                Evidence::FloatingOrbs => {
                    // Floating orbs are visible if lights are off and we are near the breach
                    let dist = player_pos.distance(&haunt_state.breach_pos);
                    let lux = board_data
                        .light_field
                        .get(player_pos.to_board_position().ndidx())
                        .map(|l| l.lux)
                        .unwrap_or(0.0);

                    if lux < 0.1 && dist < 5.0 && haunt_state.evidences.contains(&evidence) {
                        clarity = (5.0 - dist) / 5.0;
                    }
                }
                _ => {
                    // For other evidences, we use ghost_warning_intensity as a proxy for now
                    // but only if the ghost actually has that evidence.
                    if haunt_state.evidences.contains(&evidence) {
                        clarity = haunt_state.ghost_warning_intensity;
                    }
                }
            }

            evidence_readings.report_clarity(evidence, clarity, current_time, delta_time);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        update_current_evidence_readings_from_player_perception_system,
    );
}
