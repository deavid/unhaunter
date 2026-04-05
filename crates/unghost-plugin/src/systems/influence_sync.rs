use bevy::prelude::*;
use unghost_core::components::logic::ghost_influence::{GhostInfluence, InfluenceType};
use unlight_core::spectral::{SpectralInfluence, SpectralInfluenceType};
use unmetrics_core::metrics::SendMetric;

use crate::metrics;

pub(crate) fn ghost_influence_visual_sync(mut q: Query<(&GhostInfluence, &mut SpectralInfluence)>) {
    let measure = metrics::GHOST_INFLUENCE_VISUAL_SYNC.time_measure();
    for (gi, mut si) in q.iter_mut() {
        si.charge_value = gi.charge_value;
        si.influence_type = match gi.influence_type {
            InfluenceType::Attractive => SpectralInfluenceType::Attractive,
            InfluenceType::Repulsive => SpectralInfluenceType::Repulsive,
        };
    }
    measure.end_ms();
}
