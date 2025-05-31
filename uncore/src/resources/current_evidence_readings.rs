use crate::types::evidence::Evidence;
use bevy::prelude::*;
use enum_iterator::all;

#[derive(Debug, Default, Clone, Copy)]
pub struct EvidenceReading {
    pub clarity: f32,
    pub last_updated_time: f64,
    /// The entity that last updated this reading. Unused for now, this could be used to know which visual indicator was responsible for the reading.
    pub _source_gear_entity: Option<Entity>,
}

#[derive(Resource, Debug, Clone)]
pub struct CurrentEvidenceReadings {
    pub readings: Vec<EvidenceReading>,
}

impl Default for CurrentEvidenceReadings {
    fn default() -> Self {
        Self {
            readings: vec![EvidenceReading::default(); all::<Evidence>().count()],
        }
    }
}

impl CurrentEvidenceReadings {
    pub fn report_clarity(
        &mut self,
        evidence: Evidence,
        reported_clarity_from_this_gear: f32,
        current_game_time_secs: f64,
        delta_time_secs: f32, // Bevy's time.delta_secs()
    ) {
        let Some(reading) = self.readings.get_mut(evidence as usize) else {
            warn!(
                "Evidence index out of bounds in CurrentEvidenceReadings: {:?}",
                evidence
            );
            return;
        };

        let target_clarity = reported_clarity_from_this_gear.clamp(0.0, 1.0);

        if target_clarity <= reading.clarity {
            return;
        }

        // Ramping up
        const RAMP_UP_DURATION_SECONDS: f32 = 5.0; // Time to go from current to target
        let increase_this_frame = (1.0 / RAMP_UP_DURATION_SECONDS) * delta_time_secs;
        reading.clarity = (reading.clarity + increase_this_frame).min(target_clarity);

        reading.last_updated_time = current_game_time_secs;
    }

    pub fn get_reading(&self, evidence: Evidence) -> Option<&EvidenceReading> {
        self.readings.get(evidence as usize)
    }

    pub fn is_clearly_visible(&self, evidence: Evidence, clarity_threshold: f32) -> bool {
        self.get_reading(evidence)
            .is_some_and(|r| r.clarity >= clarity_threshold)
    }
}
