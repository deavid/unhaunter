use crate::resources::current_evidence_readings::CurrentEvidenceReadings;
use crate::types::evidence::Evidence;
use bevy::prelude::*;
use enum_iterator::all;

const DECAY_START_THRESHOLD_SECONDS: f64 = 0.1; // 100ms
const FULL_DECAY_DURATION_SECONDS: f64 = 10.0; // Time for clarity to go from 1.0 to 0.0 if not updated

fn decay_evidence_clarity_system(
    mut evidence_readings: ResMut<CurrentEvidenceReadings>,
    time: Res<Time>,
    mut last_report: Local<f64>,
) {
    let current_game_time = time.elapsed_secs_f64();
    let delta_seconds_for_decay = time.delta_secs_f64();

    let can_report = current_game_time - *last_report > 5.0;

    for evidence_type in all::<Evidence>() {
        // Iterate through all defined evidence types
        let idx = evidence_type as usize;
        if idx < evidence_readings.readings.len() {
            let reading = &mut evidence_readings.readings[idx];

            if reading.clarity > 0.0 {
                if can_report {
                    *last_report = current_game_time;
                    info!(
                        "Evidence clarity for {:?}: {:.1}%",
                        evidence_type,
                        reading.clarity * 100.0
                    );
                }
                // Only decay if there's clarity to begin with
                let time_since_last_update = current_game_time - reading.last_updated_time;

                if time_since_last_update > DECAY_START_THRESHOLD_SECONDS {
                    let decay_rate_per_second = 1.0 / FULL_DECAY_DURATION_SECONDS;
                    let decay_this_frame = decay_rate_per_second * delta_seconds_for_decay;

                    reading.clarity -= decay_this_frame as f32;
                    reading.clarity = reading.clarity.clamp(0.0, 1.0);

                    if reading.clarity < 0.001 {
                        // If effectively zero
                        reading.clarity = 0.0;
                        reading._source_gear_entity = None;
                    }
                    // We don't touch last_updated_time here; decay doesn't count as an "update" by a source.
                }
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, decay_evidence_clarity_system);
}
