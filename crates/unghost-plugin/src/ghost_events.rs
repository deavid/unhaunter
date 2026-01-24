use bevy::prelude::*;
use unbehavior::behavior::Behavior;
use unevents_core::events::board_topology_rebuild::BoardTopologyToRebuild;
use unevents_core::events::ghost_interaction::GhostInteractionEvent;

// NOTE: Old GhostEvent enum removed - replaced by GhostInteractionEvent system
// The new system provides more sophisticated ghost AI with personality-driven behavior
// GhostInteractionEvent and GhostInteractionType have been moved to uncore::events::ghost_interaction

// Timer component for flickering lights - maintained for legacy light flicker effects
#[derive(Component)]
pub(crate) struct FlickerTimer {
    pub timer: Timer,
}

impl Default for FlickerTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

fn update_flicker_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut q_lights: Query<(Entity, &mut FlickerTimer, &mut Behavior)>,
    mut ev_bdr: MessageWriter<BoardTopologyToRebuild>,
) {
    for (entity, mut flicker_timer, mut behavior) in q_lights.iter_mut() {
        flicker_timer.timer.tick(time.delta());
        if flicker_timer.timer.is_finished() {
            // Reset the light to its original state using the public method
            behavior.p.light.flickering = false;
            commands.entity(entity).remove::<FlickerTimer>();
            ev_bdr.write(BoardTopologyToRebuild {
                lighting: true,
                collision: true,
            });
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_message::<GhostInteractionEvent>();
    app.add_systems(
        Update,
        (
            // Old trigger_ghost_events system removed - replaced by personality-driven selection
            update_flicker_timers,
        ),
    );

    // Initialize GIS (Ghost Interaction System) module
    crate::systems::gis::setup::app_setup(app);
}
