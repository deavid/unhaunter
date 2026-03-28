use bevy::prelude::*;

use unghost_core::components::ghost_sprite::GhostSprite;
use unghost_core::events::{EvidenceClarityThresholdCrossed, GhostActualTypeChanged};
use unghost_core::resources::current_evidence_readings::{
    CurrentEvidenceReadings, HIGH_CLARITY_THRESHOLD,
};
use unghost_core::types::evidence::Evidence;

/// Monitors `CurrentEvidenceReadings` for threshold crossings and emits events on rising/falling edges.
///
/// Runs without a state gate so events are captured even when the player is not in the truck.
fn push_clarity_threshold_events(
    evidence_readings: Res<CurrentEvidenceReadings>,
    mut writer: MessageWriter<EvidenceClarityThresholdCrossed>,
    mut prev_above: Local<[bool; 8]>,
) {
    for evidence in Evidence::all() {
        let idx = evidence as usize;
        let clarity = evidence_readings
            .get_reading(evidence)
            .map_or(0.0, |r| r.clarity);
        let now_above = clarity >= HIGH_CLARITY_THRESHOLD;
        if now_above != prev_above[idx] {
            prev_above[idx] = now_above;
            writer.write(EvidenceClarityThresholdCrossed {
                evidence,
                above_threshold: now_above,
            });
        }
    }
}

/// Emits `GhostActualTypeChanged` whenever the `GhostSprite` component changes.
///
/// Runs without a state gate; `GhostSprite` is spawned at mission start, before the truck opens.
fn push_ghost_type_changed(
    q_ghost: Query<&GhostSprite, Changed<GhostSprite>>,
    mut writer: MessageWriter<GhostActualTypeChanged>,
) {
    for gs in q_ghost.iter() {
        writer.write(GhostActualTypeChanged {
            ghost_type: gs.class,
        });
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (push_clarity_threshold_events, push_ghost_type_changed),
    );
}
