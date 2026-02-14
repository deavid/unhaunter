use crate::performance_report::report_performance;
use bevy::prelude::*;
use unmetrics_core::metrics::receive_data;

pub struct UnhaunterMetricsPlugin;

impl Plugin for UnhaunterMetricsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, receive_data)
            .add_systems(Update, report_performance);
    }
}
