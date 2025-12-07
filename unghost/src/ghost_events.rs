use bevy::prelude::*;
use rand::Rng;
use uncore::behavior;
use uncore::events::board_data_rebuild::BoardDataToRebuild;
use uncore::events::ghost_interaction::GhostInteractionEvent;
use uncore::random_seed;

// NOTE: Old GhostEvent enum removed - replaced by GhostInteractionEvent system
// The new system provides more sophisticated ghost AI with personality-driven behavior
// GhostInteractionEvent and GhostInteractionType have been moved to uncore::events::ghost_interaction

// Timer component for flickering lights - maintained for legacy light flicker effects
#[derive(Component)]
pub struct FlickerTimer {
    pub timer: Timer,
    pub flash_count: usize,
    pub max_flashes: usize,
}

impl Default for FlickerTimer {
    fn default() -> Self {
        let mut rng = random_seed::rng();
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            flash_count: 0,
            max_flashes: rng.random_range(3..=12), // 3-12 flashes
        }
    }
}

fn update_flicker_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut q_lights: Query<(Entity, &mut FlickerTimer, &mut behavior::Behavior)>,
    mut ev_bdr: MessageWriter<BoardDataToRebuild>,
) {
    for (entity, mut flicker_timer, mut behavior) in q_lights.iter_mut() {
        flicker_timer.timer.tick(time.delta());
        if flicker_timer.timer.is_finished() {
            // Reset the light to its original state using the public method
            behavior.p.light.flickering = false;
            commands.entity(entity).remove::<FlickerTimer>();
            ev_bdr.write(BoardDataToRebuild {
                lighting: true,
                collision: true,
            });
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_message::<GhostInteractionEvent>();
    app.add_systems(
        Update,
        (
            // Old trigger_ghost_events system removed - replaced by personality-driven selection
            update_flicker_timers,
        ),
    );

    // Initialize GIS (Ghost Interaction System) module
    crate::systems::gis::app_setup(app);
}
