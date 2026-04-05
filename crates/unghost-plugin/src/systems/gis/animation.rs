use bevy::prelude::*;
use unmetrics_core::metrics::SendMetric;
use unspatial_core::position::Position;

use crate::metrics;

use unghost_core::components::logic::interaction::{InteractionMotion, Locked};

/// Registers animation systems with the Bevy app
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        bevy::prelude::Update,
        (tween_animation_system, door_lock_timer_system),
    );
}

/// System that updates entity positions based on authoritative interaction motion.
fn tween_animation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut q_tweens: Query<(Entity, &mut Position, &InteractionMotion)>,
) {
    let measure = metrics::GIS_TWEEN_ANIMATION.time_measure();
    let current_secs = time.elapsed_secs_f64();
    for (entity, mut position, tween) in q_tweens.iter_mut() {
        *position = tween.current_position(current_secs);
        if tween.is_finished(current_secs) {
            commands.entity(entity).remove::<InteractionMotion>();
        }
    }
    measure.end_ms();
}

/// System that removes the `Locked` component from doors when their lock duration expires.
fn door_lock_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    q_locked: Query<(Entity, &Locked)>,
) {
    let measure = metrics::GIS_DOOR_LOCK_TIMER.time_measure();
    let current_secs = time.elapsed_secs_f64();
    for (entity, locked) in q_locked.iter() {
        if locked.is_finished(current_secs) {
            commands.entity(entity).remove::<Locked>();
        }
    }
    measure.end_ms();
}
