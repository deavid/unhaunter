use bevy::prelude::*;
use unmetrics_core::metrics::SendMetric;
use unreplicon_core::messages::MovableMotionBroadcast;
use unspatial_core::position::Position;

use crate::metrics;

use crate::components::interaction::{Locked, MotionBlur, Tween, TweenEase};

/// Registers animation systems with the Bevy app
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        bevy::prelude::Update,
        (tween_animation_system, door_lock_timer_system),
    );
    app.add_systems(
        bevy::prelude::Update,
        apply_remote_movable_motion.run_if(unreplicon_core::resources::is_pure_client),
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
    let measure = metrics::GIS_TWEEN_ANIMATION.time_measure();
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
        if tween.timer.is_finished() {
            commands.entity(entity).remove::<Tween>();

            // For nudge animations, we might want to add a small return animation
            // but for now we'll keep it simple and just end at the nudged position
        }
    }
    measure.end_ms();
}

/// Client (join mode only): receive a `MovableMotionBroadcast` from the server
/// and replay the same tween animation on the matching local entity.
///
/// The entity is identified by its `Entity` handle, which is automatically
/// mapped from server to client by `bevy_replicon`.
fn apply_remote_movable_motion(
    mut reader: MessageReader<MovableMotionBroadcast>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let ease_fn = match msg.ease {
            1 => TweenEase::ParabolicArc,
            2 => TweenEase::SineEaseOut,
            _ => TweenEase::Linear,
        };
        commands.entity(msg.entity).insert(Tween {
            start_pos: Position {
                x: msg.start[0],
                y: msg.start[1],
                z: msg.start[2],
                visual_priority: msg.start[3],
            },
            end_pos: Position {
                x: msg.end[0],
                y: msg.end[1],
                z: msg.end[2],
                visual_priority: msg.end[3],
            },
            timer: Timer::from_seconds(msg.duration, TimerMode::Once),
            ease_fn,
        });
    }
}
///
/// This system ticks down the locked timer on doors and removes the lock when
/// the timer expires, allowing the door to be used normally again.
fn door_lock_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut q_locked: Query<(Entity, &mut Locked)>,
) {
    let measure = metrics::GIS_DOOR_LOCK_TIMER.time_measure();
    for (entity, mut locked) in q_locked.iter_mut() {
        // Tick the lock timer
        locked.0.tick(time.delta());

        // If the timer is finished, remove the Locked component
        if locked.0.is_finished() {
            commands.entity(entity).remove::<Locked>();
        }
    }
    measure.end_ms();
}
