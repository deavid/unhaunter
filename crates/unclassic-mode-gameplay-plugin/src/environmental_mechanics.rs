use bevy::prelude::*;
use unbehavior_core::behavior::Behavior;
use unbehavior_core::components::Light;
use unbehavior_core::state::TileState;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unghost_core::events::{GhostInteractionEvent, GhostInteractionType};
use uninteraction_core::events::InteractionExecutionType;
use uninteraction_core::interaction::ExecuteInteractionEvent;
use unspatial_core::position::Position;

/// Cooldown timer to prevent rapid re-tripping of the breaker
#[derive(Resource)]
#[allow(dead_code)]
struct FuseBoxCooldownTimer(Timer);

impl Default for FuseBoxCooldownTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(10.0, TimerMode::Once))
    }
}

/// System that monitors electrical load and trips the breaker when too many lights are on
///
/// This system implements an environmental mechanic where the electrical system
/// has limits. If the player turns on too many lights simultaneously, the main
/// breaker will automatically trip, plunging the map into darkness.
fn fuse_box_overload_system(
    time: Res<Time>,
    difficulty: Res<CurrentDifficulty>,
    mut cooldown_timer: Local<Option<Timer>>,
    q_lights: Query<&Behavior, With<Light>>,
    q_breakers: Query<(Entity, &Behavior), Without<Light>>,
    mut ev_ghost_interaction: MessageWriter<GhostInteractionEvent>,
) {
    // Initialize cooldown timer if it doesn't exist
    if cooldown_timer.is_none() {
        *cooldown_timer = Some(Timer::from_seconds(5.0, TimerMode::Once));
    }

    // Tick the cooldown timer
    if let Some(timer) = cooldown_timer.as_mut() {
        timer.tick(time.delta());

        // If still in cooldown, don't check for overload
        if !timer.is_finished() {
            return;
        }
    }

    // Count total house-powered lights and lights that are currently on
    let mut total_lights = 0;
    let mut lights_on = 0;

    for behavior in q_lights.iter() {
        if behavior.can_emit_light() && behavior.p.is_house_powered {
            total_lights += 1;
            if behavior.p.light.light_emission_enabled {
                lights_on += 1;
            }
        }
    }

    // Don't check if there are no lights (avoid division by zero)
    if total_lights == 0 {
        return;
    }

    // Calculate the percentage of lights that are on
    let lights_on_percentage = lights_on as f32 / total_lights as f32;

    // Base overload threshold (60% of lights on)
    let base_threshold = 0.6;

    // Adjust threshold based on difficulty
    // Higher difficulty = easier to overload (lower threshold)
    let threshold =
        base_threshold * (2.0 - difficulty.0.ghost_interaction_frequency().clamp(0.5, 2.0));

    // Check if we've exceeded the threshold
    // Minimum 6-light bonus: Only check for overload if more than 6 lights are actually on.
    if lights_on > 6 && lights_on_percentage > threshold {
        // Find a breaker to trip - look for breakers that are currently "On"
        for (breaker_entity, breaker_behavior) in q_breakers.iter() {
            // Check if this is actually a breaker and if it's currently on
            if breaker_behavior.p.is_breaker && matches!(breaker_behavior.state(), TileState::On) {
                // Dispatch a trip breaker event
                ev_ghost_interaction.write(GhostInteractionEvent {
                    target: breaker_entity,
                    interaction_type: GhostInteractionType::TripBreaker,
                    destination: None,
                });

                // Reset the cooldown timer
                if let Some(timer) = cooldown_timer.as_mut() {
                    *timer = Timer::from_seconds(30.0, TimerMode::Once);
                }

                info!(
                    "Fuse box overload! {}/{} lights were on ({:.1}% > {:.1}% threshold). Tripping breaker.",
                    lights_on,
                    total_lights,
                    lights_on_percentage * 100.0,
                    threshold * 100.0
                );

                // Only trip one breaker
                break;
            }
        }
    }
}

/// Helper system to initialize the fuse box overload timer resource
fn initialize_fuse_box_system(mut commands: Commands) {
    commands.insert_resource(FuseBoxCooldownTimer::default());
}

/// System that ensures all breakers in the level share the same state.
///
/// If one breaker is flipped (by player or ghost), all other breakers
/// will automatically flip to match its state.
fn breaker_sync_system(
    q_changed: Query<&Behavior, (With<Position>, Changed<Behavior>)>,
    q_all: Query<(Entity, &Position, &Behavior)>,
    mut ev_interaction: MessageWriter<ExecuteInteractionEvent>,
) {
    // Check if any breaker changed state this frame
    let target_state = q_changed
        .iter()
        .filter(|b| b.p.is_breaker)
        .map(|b| b.state())
        .next();

    let Some(state) = target_state else {
        return;
    };

    // Synchronize all other breakers to match this state
    for (entity, _pos, behavior) in q_all.iter() {
        if behavior.p.is_breaker && behavior.state() != state {
            ev_interaction.write(ExecuteInteractionEvent {
                entity,
                ietype: InteractionExecutionType::ChangeState,
                force_tuid: None,
            });
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (fuse_box_overload_system, breaker_sync_system)
            .run_if(in_state(untypes_core::states::SimulationState::Ready))
            .run_if(resource_exists::<untypes_core::roles::AuthorityRole>),
    );
    app.add_systems(Startup, initialize_fuse_box_system);
}
