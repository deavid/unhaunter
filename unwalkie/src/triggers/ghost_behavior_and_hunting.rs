use bevy::{prelude::*, time::Stopwatch};
use uncore::types::gear_kind::GearKind;
use uncore::{
    components::{
        board::position::Position, ghost_sprite::GhostSprite, player::Hiding,
        player_sprite::PlayerSprite,
    },
    states::{AppState, GameState},
};
use ungear::components::playergear::PlayerGear;
use unwalkiecore::{WalkieEvent, WalkiePlay};

const NO_EVASION_TIMER_SECONDS: f32 = 4.0;
const NO_EVASION_MAX_DISTANCE: f32 = 1.0; // Max distance player can move to still be considered "not evaded"

fn trigger_hunt_warning_no_player_evasion_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
    q_player: Query<(&Position, Option<&Hiding>, &PlayerGear), With<PlayerSprite>>,
    q_ghost: Query<&GhostSprite>,
    mut warning_timer: Local<Option<Stopwatch>>,
    mut player_pos_at_warning: Local<Option<Position>>,
) {
    // 1. System Run Condition Checks
    if *app_state.get() != AppState::InGame || *game_state.get() != GameState::None {
        if warning_timer.is_some() {
            *warning_timer = None;
            *player_pos_at_warning = None;
        }
        return;
    }

    let Ok((player_current_pos, maybe_hiding, player_gear)) = q_player.get_single() else {
        if warning_timer.is_some() {
            *warning_timer = None;
            *player_pos_at_warning = None;
        }
        return;
    };

    // Check if player has RepellentFlask in inventory (hands or general inventory)
    let has_repellent = player_gear.left_hand.kind == GearKind::RepellentFlask
        || player_gear.right_hand.kind == GearKind::RepellentFlask
        || player_gear
            .inventory
            .iter()
            .any(|item| item.kind == GearKind::RepellentFlask);

    if has_repellent {
        if warning_timer.is_some() {
            *warning_timer = None;
            *player_pos_at_warning = None;
        }
        return;
    }

    let is_player_hiding = maybe_hiding.is_some();
    let mut is_hunt_warning_active_for_any_ghost = false;
    for ghost_sprite in q_ghost.iter() {
        if ghost_sprite.hunt_warning_active {
            is_hunt_warning_active_for_any_ghost = true;
            break;
        }
    }

    // 3. Conditions for starting/resetting the timer
    if is_hunt_warning_active_for_any_ghost && !is_player_hiding {
        if warning_timer.is_none() {
            *warning_timer = Some(Stopwatch::new());
            *player_pos_at_warning = Some(*player_current_pos);
        }
    } else if warning_timer.is_some() {
        *warning_timer = None;
        *player_pos_at_warning = None;
    }

    // 4. Conditions for triggering the event
    if let Some(ref mut stopwatch) = *warning_timer {
        stopwatch.tick(time.delta());

        if stopwatch.elapsed_secs() > NO_EVASION_TIMER_SECONDS {
            if !is_player_hiding {
                // Re-check hiding status
                if let Some(initial_pos) = *player_pos_at_warning {
                    if player_current_pos.distance(&initial_pos) < NO_EVASION_MAX_DISTANCE {
                        // FIXME: Verification needed: Not sure if this trigger actually fires. Don't recall it having fired in testing.
                        if walkie_play.set(
                            WalkieEvent::HuntWarningNoPlayerEvasion,
                            time.elapsed_secs_f64(),
                        ) {
                            *warning_timer = None;
                            *player_pos_at_warning = None;
                        }
                    } else {
                        // Player moved enough, reset timer
                        *warning_timer = None;
                        *player_pos_at_warning = None;
                    }
                } else {
                    // Should not happen if timer is Some, but good to reset
                    *warning_timer = None;
                    *player_pos_at_warning = None;
                }
            } else {
                // Player started hiding, reset timer
                *warning_timer = None;
                *player_pos_at_warning = None;
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, trigger_hunt_warning_no_player_evasion_system);
}
