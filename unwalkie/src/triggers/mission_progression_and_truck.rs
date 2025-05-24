use bevy::{prelude::*, time::Stopwatch};
use bevy_persistent::Persistent;
use uncore::{
    components::{
        ghost_breach::GhostBreach, ghost_sprite::GhostSprite, player_sprite::PlayerSprite,
    },
    states::{AppState, GameState},
};
use unprofile::PlayerProfileData;
// Assuming a UI state resource that indicates interaction with truck tabs
// This is a placeholder; actual resource path/name might differ.
// use untruck::ui_state::TruckUiInteractionState; // Placeholder, replace with actual if known
use unwalkiecore::{WalkieEvent, WalkiePlay};

const LINGER_DURATION_SECONDS: f32 = 45.0;

// --- New Resource for tracking loadout interaction ---
#[derive(Resource, Default, Debug)]
struct LoadoutInteractionTracker {
    interacted_this_session: bool,
}
// ---

fn trigger_all_objectives_met_reminder_system(
    mut walkie_play: ResMut<WalkiePlay>,
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    q_ghost: Query<Entity, With<GhostSprite>>,
    q_breach: Query<Entity, With<GhostBreach>>,
    mut linger_timer: Local<Option<Stopwatch>>,
) {
    // System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::Truck {
        if linger_timer.is_some() {
            *linger_timer = None;
        }
        return;
    }

    let ghost_expelled = q_ghost.is_empty();
    let breach_sealed = q_breach.is_empty();

    if ghost_expelled && breach_sealed {
        if linger_timer.is_none() {
            *linger_timer = Some(Stopwatch::new());
        }
    } else {
        if linger_timer.is_some() {
            *linger_timer = None;
        }
        return;
    }

    if let Some(timer) = linger_timer.as_mut() {
        timer.tick(time.delta());
        if timer.elapsed_secs() >= LINGER_DURATION_SECONDS
            && walkie_play.set(
                WalkieEvent::AllObjectivesMetReminderToEndMission,
                time.elapsed_secs_f64(),
            )
        {
            *linger_timer = None; // Reset timer after firing
        }
    }
}

// --- New System 1: Track Loadout Interaction ---
fn track_loadout_interaction_system(
    mut tracker: ResMut<LoadoutInteractionTracker>,
    // This is an assumed resource. If the actual resource/event for UI interaction is different,
    // this query needs to be adjusted. For example, listening to an EventWriter<LoadoutTabClickedEvent>.
    // Or, if TruckUIState is a component on a UI entity: Query<&TruckUIState, Changed<TruckUIState>>
    // truck_ui_state: Option<Res<TruckUiInteractionState>>, // Making it Option<> to avoid crash if not found
    game_state: Res<State<GameState>>, // To ensure it only runs in Truck
    keyboard_input: Res<ButtonInput<KeyCode>>, // Temporary: Using a key press as a proxy for UI interaction
    player_profile: Res<Persistent<PlayerProfileData>>, // To check if it's not the first mission
) {
    if *game_state.get() != GameState::Truck {
        return;
    }

    // Placeholder for actual UI interaction detection
    // For now, let's assume pressing "L" (for Loadout) in the truck means interaction.
    // This is a temporary workaround.
    // A real implementation would listen to UI events or check a UI state resource.
    if keyboard_input.just_pressed(KeyCode::KeyL) {
        // Replace KeyCode::L with actual interaction logic
        if player_profile.statistics.total_missions_completed > 0 {
            // Only track after the very first mission
            tracker.interacted_this_session = true;
            // info!("[WalkieDebug] Loadout interaction (proxy via L key) detected!");
        }
    }

    // Example with a hypothetical TruckUiInteractionState resource:
    // if let Some(ui_state) = truck_ui_state {
    //     if ui_state.loadout_tab_was_just_opened { // Replace with actual field/logic
    //         tracker.interacted_this_session = true;
    //         // info!("[WalkieDebug] Loadout tab interaction detected!");
    //     }
    // } else {
    //     // warn!("[WalkieDebug] TruckUiInteractionState resource not found. Loadout interaction tracking might be inaccurate.");
    // }
}

// --- New System 2: Trigger on Leaving Truck ---
fn trigger_player_leaves_truck_without_changing_loadout_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut prev_game_state: Local<GameState>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_profile: Res<Persistent<PlayerProfileData>>,
    mut tracker: ResMut<LoadoutInteractionTracker>,
    _player_q: Query<Entity, With<PlayerSprite>>,
) {
    if *app_state.get() != AppState::InGame {
        *prev_game_state = *game_state.get(); // Keep prev_game_state updated
        return;
    }

    let current_gs = *game_state.get();
    let previous_gs = *prev_game_state;
    *prev_game_state = current_gs; // Update for next frame

    // Player enters the truck: Reset interaction flag
    if current_gs == GameState::Truck && previous_gs != GameState::Truck {
        // info!("[WalkieDebug] Player entered truck. Resetting loadout interaction flag.");
        tracker.interacted_this_session = false;
    }
    // Player leaves the truck
    else if current_gs == GameState::None && previous_gs == GameState::Truck {
        // info!("[WalkieDebug] Player left truck. Missions completed: {}, Interacted with loadout: {}",
        //     player_profile.statistics.total_missions_completed,
        //     tracker.interacted_this_session);

        // Only trigger if player has completed at least one mission (i.e., not the very first time playing)
        if player_profile.statistics.total_missions_completed >= 1
            && !tracker.interacted_this_session
        {
            // info!("[WalkieDebug] Conditions met for PlayerLeavesTruckWithoutChangingLoadout. Attempting to set event.");
            walkie_play.set(
                WalkieEvent::PlayerLeavesTruckWithoutChangingLoadout,
                time.elapsed_secs_f64(),
            );
        }
        // Reset for the next truck session regardless of whether the hint fired or conditions were met
        tracker.interacted_this_session = false;
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<LoadoutInteractionTracker>() // Initialize the new resource
        .add_systems(Update, trigger_all_objectives_met_reminder_system)
        .add_systems(
            Update,
            track_loadout_interaction_system.run_if(in_state(GameState::Truck)), // Ensure runs only in Truck
        )
        .add_systems(
            Update,
            trigger_player_leaves_truck_without_changing_loadout_system, // Runs always to catch state changes
        );
}
