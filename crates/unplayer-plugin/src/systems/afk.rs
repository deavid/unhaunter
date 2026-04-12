use bevy::prelude::*;
use unspatial_core::position::Position;
use unspatial_core::direction::Direction;
use untruck_core::components::in_truck::InTruck;
use unplayer_core::components::{PlayerInactive, PlayerSprite};
use unreplicon_core::resources::AuthorityRole;

const AFK_TIMEOUT_SECS: f32 = 120.0;

#[derive(Component, Debug)]
pub(crate) struct AfkTracker {
    pub last_position: Position,
    pub last_direction: Option<Direction>,
    pub idle_time: f32,
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        track_afk_system.run_if(resource_exists::<AuthorityRole>),
    );
}

pub(crate) fn track_afk_system(
    mut commands: Commands,
    time: Res<Time>,
    mut q_players: Query<(
        Entity,
        &Position,
        Option<&Direction>,
        Option<&mut AfkTracker>,
        Has<PlayerInactive>,
        Has<InTruck>,
    ), With<PlayerSprite>>,
) {
    let dt = time.delta_secs();

    for (entity, position, direction, tracker_opt, is_inactive, in_truck) in q_players.iter_mut() {
        if in_truck {
            // Players in the truck are never AFK
            if is_inactive {
                commands.entity(entity).remove::<PlayerInactive>();
            }
            if tracker_opt.is_some() {
                commands.entity(entity).remove::<AfkTracker>();
            }
            continue;
        }

        if let Some(mut tracker) = tracker_opt {
            let dir_changed = match (tracker.last_direction, direction) {
                (Some(last_dir), Some(current_dir)) => last_dir != *current_dir,
                (None, Some(_)) | (Some(_), None) => true,
                (None, None) => false,
            };

            // Check if they moved or rotated
            if tracker.last_position != *position || dir_changed {
                // Active!
                tracker.last_position = *position;
                tracker.last_direction = direction.copied();
                tracker.idle_time = 0.0;
                if is_inactive {
                    commands.entity(entity).remove::<PlayerInactive>();
                }
            } else {
                // Idle
                tracker.idle_time += dt;
                if tracker.idle_time >= AFK_TIMEOUT_SECS && !is_inactive {
                    debug!("Player {:?} marked as AFK", entity);
                    commands.entity(entity).insert(PlayerInactive);
                }
            }
        } else {
            // Setup tracker
            commands.entity(entity).insert(AfkTracker {
                last_position: *position,
                last_direction: direction.copied(),
                idle_time: 0.0,
            });
        }
    }
}
