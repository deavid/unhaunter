use bevy::{prelude::*, time::Stopwatch};
use bevy_persistent::Persistent;
use uncore::{
    components::{
        game_config::GameConfig, ghost_breach::GhostBreach, ghost_sprite::GhostSprite,
        player_sprite::PlayerSprite,
    },
    states::{AppState, GameState},
};
use ungear::components::playergear::PlayerGear;
use unprofile::PlayerProfileData;
use unwalkiecore::{WalkieEvent, WalkiePlay};

const LINGER_DURATION_SECONDS: f32 = 45.0;

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
        // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
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

fn trigger_player_leaves_truck_without_changing_loadout_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut prev_game_state: Local<GameState>,
    mut walkie_play: ResMut<WalkiePlay>,
    player_profile: Res<Persistent<PlayerProfileData>>,
    difficulty: Res<uncore::difficulty::CurrentDifficulty>,
    player_gear_q: Query<(&PlayerSprite, &PlayerGear)>,
    game_config: Res<GameConfig>,
) {
    if *app_state.get() != AppState::InGame {
        *prev_game_state = *game_state.get();
        return;
    }

    // Only trigger if van auto-open is enabled in difficulty settings
    if !difficulty.0.van_auto_open {
        return;
    }

    let current_gs = *game_state.get();
    let previous_gs = *prev_game_state;
    *prev_game_state = current_gs;

    // Player leaves the truck (transitions from Truck to None)
    if current_gs == GameState::None && previous_gs == GameState::Truck {
        // Only trigger if player has completed at least one mission (not first time)
        if player_profile.statistics.total_missions_completed >= 1 {
            // Check if the current player has empty right hand
            for (player_sprite, player_gear) in player_gear_q.iter() {
                if player_sprite.id == game_config.player_id && player_gear.empty_right_handed() {
                    walkie_play.set(
                        WalkieEvent::PlayerLeavesTruckWithoutChangingLoadout,
                        time.elapsed_secs_f64(),
                    );
                    break; // Only need to check once per player
                }
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, trigger_all_objectives_met_reminder_system)
        .add_systems(
            Update,
            trigger_player_leaves_truck_without_changing_loadout_system,
        );
}
