use bevy::{prelude::*, time::Stopwatch};
use unboard_core::resources::roomdb::RoomTopology;
use uncommon_states_core::UIContextState;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::types::gear::kind::GearKind;
use unghost_core::resources::signals::GhostHuntSignals;
use unplayer_core::components::MainPlayer;
use unspatial_core::position::Position;
use unwalkie_core::events::walkie_types::WalkieEvent;
use unwalkie_core::messages::ProposeWalkieEvent;
use unwalkie_core::resources::WalkiePlay;

/// Reminds the player to pick up equipment if they enter the location without any gear during the tutorial.
/// Only triggers if the player is in the game, not in the truck, and has accessed the truck at least once.
/// Uses a stopwatch to avoid spamming the reminder and only warns within the first minute inside.
fn player_forgot_equipment(
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    qp: Query<(&Position, &PlayerGear), With<MainPlayer>>,
    room_topology: Res<RoomTopology>,
    mut stopwatch: Local<Stopwatch>,
    app_state: Res<State<UIContextState>>,
    time: Res<Time>,
) {
    if app_state.get() != &UIContextState::InGame {
        // We want to play this only when the player is in the game.
        stopwatch.reset();
        return;
    }
    if !walkie_play.truck_accessed {
        // The player didn't had a chance to grab stuff, so don't tell them to.
        stopwatch.reset();
        return;
    }
    // Find the active player's position
    let mut any_player_inside_without_gear = false;
    for (player_pos, player_gear) in qp.iter() {
        let player_bpos = player_pos.to_board_position();

        if room_topology.room_tiles.get(&player_bpos).is_some() {
            if player_gear.right_hand.is_some() {
                // At least one player has an item, no need to remind anyone.
                walkie_play.mark(WalkieEvent::GearInVan, time.elapsed_secs_f64());
                stopwatch.reset();
                return;
            }
            any_player_inside_without_gear = true;
        }
    }

    if !any_player_inside_without_gear {
        stopwatch.reset();
        return;
    }

    stopwatch.tick(time.delta());
    if stopwatch.elapsed().as_secs_f32() < 1.0 {
        // Wait before reminding the player.
        return;
    }
    if stopwatch.elapsed().as_secs_f32() > 60.0 {
        // Too much time inside the location, we want to warn mainly when it crosses the main door.
        return;
    }
    crate::triggers::net::walkie_set_or_propose(
        WalkieEvent::GearInVan,
        time.elapsed_secs_f64(),
        &mut walkie_play,
        &mut ev_propose,
    );
}

/// Warns the player via walkie-talkie when the ghost is close to starting a hunt in the tutorial.
/// Only triggers if the player is inside the location and the ghost's rage is high but not yet hunting.
fn ghost_near_hunt(
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    qp: Query<(&Position, &PlayerGear), With<MainPlayer>>,
    room_topology: Res<RoomTopology>,
    difficulty: Res<CurrentDifficulty>,
    hunt_signals: Res<GhostHuntSignals>,
    q_gear: Query<&GearKind>,
    time: Res<Time>,
) {
    if !difficulty.0.is_tutorial_difficulty() {
        // Not in tutorial mode, no need to tell the player.
        return;
    }
    // Find the active player's position and gear
    for (player_pos, player_gear) in qp.iter() {
        // If player has RepellentFlask, disable this system
        let check_gear = |entity: Entity| -> bool {
            if let Ok(kind) = q_gear.get(entity) {
                *kind == GearKind::RepellentFlask
            } else {
                false
            }
        };

        let has_repellent = player_gear.left_hand.map(check_gear).unwrap_or(false)
            || player_gear.right_hand.map(check_gear).unwrap_or(false)
            || player_gear.inventory.iter().any(|&e| check_gear(e));

        if has_repellent {
            return;
        }

        let player_bpos = player_pos.to_board_position();

        if room_topology.room_tiles.get(&player_bpos).is_none() {
            // Player is not inside the location, no need to tell them.
            continue;
        }
        if hunt_signals.any_near_hunt_without_warning {
            crate::triggers::net::walkie_set_or_propose(
                WalkieEvent::GhostNearHunt,
                time.elapsed_secs_f64(),
                &mut walkie_play,
                &mut ev_propose,
            );
            return;
        }
    }
}

/// Registers the above systems to the Bevy app.
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, player_forgot_equipment)
        .add_systems(Update, ghost_near_hunt);
}
