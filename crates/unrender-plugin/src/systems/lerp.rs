use bevy::prelude::*;
use unspatial_core::lerp_position::LerpPosition;
use unspatial_core::position::Position;

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(PreUpdate, advance_lerp_position);
}

fn advance_lerp_position(mut q: Query<(&Position, &mut LerpPosition)>, time: Res<Time>) {
    for (pos, mut lerp) in q.iter_mut() {
        let alpha = (lerp.speed * time.delta_secs()).min(1.0);
        lerp.current = lerp.current.lerp(pos, alpha);
    }
}
