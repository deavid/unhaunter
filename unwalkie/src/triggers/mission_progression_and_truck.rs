use bevy::{prelude::*, time::Stopwatch};
use bevy_platform::collections::HashSet;
use uncore::{
    components::{
        game_config::GameConfig, ghost_breach::GhostBreach, ghost_sprite::GhostSprite,
        player_sprite::PlayerSprite,
    },
    states::{AppState, GameState},
    types::{evidence::Evidence, gear_kind::GearKind},
};
use ungear::components::playergear::PlayerGear;
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
    difficulty: Res<uncore::difficulty::CurrentDifficulty>,
    player_gear_q: Query<(&PlayerSprite, &PlayerGear)>,
    game_config: Res<GameConfig>,
    mut exited_truck_time: Local<Option<f64>>,
    mut empty_right_handed: Local<bool>,
    mut discoverable_evidences_with_current_gear: Local<HashSet<Evidence>>,
    mut has_repellent_flask: Local<bool>,
    mut last_gear_evidences_change_time: Local<Option<f64>>,
) {
    if *app_state.get() != AppState::InGame {
        *exited_truck_time = None;
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
        *exited_truck_time = Some(time.elapsed_secs_f64());

        // Check if the current player has empty right hand
        for (player_sprite, player_gear) in player_gear_q.iter() {
            if player_sprite.id == game_config.player_id {
                // Calculate current set of discoverable evidences from player's gear
                let mut new_discoverable_evidences = HashSet::new();
                let mut new_has_repellent_flask = false;

                // Check left hand gear
                if let Ok(evidence) = Evidence::try_from(&player_gear.left_hand.kind) {
                    new_discoverable_evidences.insert(evidence);
                }
                if player_gear.left_hand.kind == GearKind::RepellentFlask {
                    new_has_repellent_flask = true;
                }

                // Check right hand gear
                if let Ok(evidence) = Evidence::try_from(&player_gear.right_hand.kind) {
                    new_discoverable_evidences.insert(evidence);
                }
                if player_gear.right_hand.kind == GearKind::RepellentFlask {
                    new_has_repellent_flask = true;
                }

                // Check inventory gear
                for gear_item in &player_gear.inventory {
                    if let Ok(evidence) = Evidence::try_from(&gear_item.kind) {
                        new_discoverable_evidences.insert(evidence);
                    }
                    if gear_item.kind == GearKind::RepellentFlask {
                        new_has_repellent_flask = true;
                    }
                }

                // Check if the set of discoverable evidences has changed OR repellent flask status changed
                if *discoverable_evidences_with_current_gear != new_discoverable_evidences
                    || *has_repellent_flask != new_has_repellent_flask
                {
                    *discoverable_evidences_with_current_gear = new_discoverable_evidences;
                    *has_repellent_flask = new_has_repellent_flask;
                    *last_gear_evidences_change_time = Some(time.elapsed_secs_f64());
                }
                *empty_right_handed = player_gear.empty_right_handed();

                break;
            }
        }
    }
    if let Some(exited_time) = *exited_truck_time {
        let cur_time = time.elapsed_secs_f64();
        if cur_time - exited_time > 60.0 {
            return;
        }
        let too_long_checking_evidence =
            (exited_time - last_gear_evidences_change_time.unwrap_or(exited_time)) > 120.0;
        let trigger = (*empty_right_handed || too_long_checking_evidence) && !*has_repellent_flask;
        if trigger
            && walkie_play.set(
                WalkieEvent::PlayerLeavesTruckWithoutChangingLoadout,
                cur_time,
            )
        {
            *exited_truck_time = None;
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
