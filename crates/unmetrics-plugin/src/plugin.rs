use bevy::prelude::*;
use unmetrics_core::metrics::receive_data;

pub struct UnmetricsPlugin;

impl Plugin for UnmetricsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, receive_data);
    }
}
