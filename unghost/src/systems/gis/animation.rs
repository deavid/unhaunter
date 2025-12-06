use bevy::prelude::*;
use uncore::components::board::position::Position;

use crate::components::interaction::{Locked, MotionBlur, Tween, TweenEase};

/// Registers animation systems with the Bevy app
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        bevy::prelude::Update,
        (tween_animation_system, door_lock_timer_system),
    );
}

/// System that updates entity positions based on their Tween component
///
/// This system handles all object movement animations for ghost interactions,
/// including throws, nudges, and haunted moves. It also handles cleanup when
/// animations complete.
fn tween_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut q_tweens: Query<(Entity, &mut Position, &mut Tween)>,
    q_motion_blur: Query<&MotionBlur>,
) {
    for (entity, mut position, mut tween) in q_tweens.iter_mut() {
        // Add motion blur for thrown objects if not already present
        if tween.ease_fn == TweenEase::ParabolicArc && q_motion_blur.get(entity).is_err() {
            commands.entity(entity).insert(MotionBlur {
                intensity: 0.0,
                previous_position: *position,
            });
        }

        // Tick the animation timer
        tween.timer.tick(time.delta());

        // Update the entity's position based on the tween progress
        *position = tween.current_position();

        // If the animation is finished, remove the Tween component
        if tween.timer.finished() {
            commands.entity(entity).remove::<Tween>();

            // For nudge animations, we might want to add a small return animation
            // but for now we'll keep it simple and just end at the nudged position
        }
    }
}

/// System that handles door lock timers and removes the Locked component when expired
///
/// This system ticks down the locked timer on doors and removes the lock when
/// the timer expires, allowing the door to be used normally again.
fn door_lock_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut q_locked: Query<(Entity, &mut Locked)>,
) {
    for (entity, mut locked) in q_locked.iter_mut() {
        // Tick the lock timer
        locked.0.tick(time.delta());

        // If the timer is finished, remove the Locked component
        if locked.0.finished() {
            commands.entity(entity).remove::<Locked>();
        }
    }
}
