use bevy::prelude::*;
use bevy_persistent::Persistent;
use unprofile::{PlayerProfileData, data::WalkieEventStats};
use unwalkiecore::{WalkiePlay, WalkieSoundState};

/// System that updates the WalkieEventStats in the player profile
/// when walkie events are played.
pub fn update_walkie_stats(
    walkie_play: Res<WalkiePlay>,
    mut player_profile: ResMut<Persistent<PlayerProfileData>>,
    time: Res<Time>,
) {
    // If a walkie event finished playing (event exists but state is None)
    if let Some(event) = &walkie_play.event {
        if walkie_play.state == Some(WalkieSoundState::Intro) {
            // Convert the event enum to a string representation
            let event_id = format!("{:?}", event);
            let now =
                player_profile.statistics.total_play_time_seconds + time.elapsed().as_secs_f64();

            // Get or create stats for this event
            let stats = player_profile
                .walkie_event_stats
                .entry(event_id.clone())
                .or_insert_with(WalkieEventStats::default);
            if now - stats.last_played_at_time < 5.0 {
                // If the event was played recently, do not update the stats
                return;
            }
            // Update stats
            stats.play_count += 1;
            stats.last_played_at_time = now;
            // Mark the profile as modified so it will be saved
            player_profile.set_changed();
        }
    }
}

// Register the system with the app
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_walkie_stats);
}
