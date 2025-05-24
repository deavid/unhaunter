use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::events::loadlevel::LevelReadyEvent;
use unprofile::PlayerProfileData;
use unwalkiecore::{WalkieEvent, WalkiePlay};

/// Helper function to parse walkie event strings back to enum variants
fn parse_walkie_event(event_str: &str) -> Option<WalkieEvent> {
    enum_iterator::all::<WalkieEvent>().find(|event| format!("{:?}", event) == event_str)
}

/// System that loads walkie event data from player profile into WalkiePlay resource
/// when a level loads. This allows the game to track which walkie events were played
/// in previous missions.
pub fn load_walkie_event_stats(
    mut ev_level_ready: EventReader<LevelReadyEvent>,
    player_profile: Res<Persistent<PlayerProfileData>>,
    mut walkie_play: ResMut<WalkiePlay>,
) {
    // Only process newly loaded levels
    if ev_level_ready.is_empty() {
        return;
    }
    for _event in ev_level_ready.read() {}

    info!("Loading walkie event stats from player profile");

    // Clear existing event count data
    walkie_play.other_mission_event_count.clear();

    // Convert string event IDs to WalkieEvent enum and populate the HashMap
    for (event_id_str, stats) in player_profile.walkie_event_stats.iter() {
        // Parse the string representation back into a WalkieEvent enum
        // This relies on the Debug representation format used when storing the events
        if let Some(walkie_event) = parse_walkie_event(event_id_str) {
            // Store the play count in the other_mission_event_count HashMap
            info!(
                "Loaded walkie event: {:?} with play count: {}",
                walkie_event, stats.play_count
            );
            walkie_play
                .other_mission_event_count
                .insert(walkie_event, stats.play_count);
        } else {
            warn!("Failed to parse walkie event ID: {}", event_id_str);
        }
    }

    info!(
        "Loaded {} walkie event stats",
        walkie_play.other_mission_event_count.len()
    );
}

// Function to register systems with the app
pub(crate) fn setup_walkie_level_systems(app: &mut App) {
    app.add_systems(Update, load_walkie_event_stats);
}
