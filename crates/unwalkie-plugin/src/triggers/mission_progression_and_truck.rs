use bevy::{prelude::*, time::Stopwatch};
use bevy_platform::collections::HashSet;
use uncommon_states_core::UIContextState;
use undifficulty_core::difficulty_settings::DifficultySettings;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::types::gear::kind::GearKind;
use unghost_core::components::logic::ghost_breach::GhostBreach;
use unghost_core::resources::signals::GhostHuntSignals;
use uninvestigation_core::evidence::Evidence;
use unplayer_core::components::MainPlayer;
use unplayer_core::components::PlayerSprite;
use untruck_core::components::in_truck::InTruck;
use unwalkie_core::events::walkie_types::WalkieEvent;
use unwalkie_core::resources::WalkiePlay;
use unwalkie_core::messages::ProposeWalkieEvent;

const LINGER_DURATION_SECONDS: f32 = 45.0;

fn trigger_all_objectives_met_reminder_system(
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    time: Res<Time>,
    app_state: Res<State<UIContextState>>,
    hunt_signals: Res<GhostHuntSignals>,
    q_breach: Query<Entity, With<GhostBreach>>,
    mut linger_timer: Local<Option<Stopwatch>>,
    qp_in_truck: Query<&InTruck, With<MainPlayer>>,
) {
    // System Run Condition Checks
    if *app_state.get() != UIContextState::InGame || qp_in_truck.is_empty() {
        if linger_timer.is_some() {
            *linger_timer = None;
        }
        return;
    }

    let ghost_expelled = !hunt_signals.any_present;
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
            && crate::triggers::net::walkie_set_or_propose(
                WalkieEvent::AllObjectivesMetReminderToEndMission,
                time.elapsed_secs_f64(),
                &mut walkie_play,
                &mut ev_propose,
            )
        {
            *linger_timer = None; // Reset timer after firing
        }
    }
}

fn trigger_player_leaves_truck_without_changing_loadout_system(
    time: Res<Time>,
    app_state: Res<State<UIContextState>>,
    q_in_truck: Query<(), (With<MainPlayer>, With<InTruck>)>,
    mut was_in_truck: Local<bool>,
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    difficulty: Res<undifficulty_core::current_difficulty::CurrentDifficulty>,
    player_gear_q: Query<(&PlayerSprite, &PlayerGear), With<MainPlayer>>,
    mut exited_truck_time: Local<Option<f64>>,
    mut empty_right_handed: Local<bool>,
    mut discoverable_evidences_with_current_gear: Local<HashSet<Evidence>>,
    mut has_repellent_flask: Local<bool>,
    mut last_gear_evidences_change_time: Local<Option<f64>>,
    q_gear: Query<&GearKind>,
) {
    if *app_state.get() != UIContextState::InGame {
        *exited_truck_time = None;
        return;
    }

    // Only trigger if van auto-open is enabled in difficulty settings
    if !difficulty.0.van_auto_open() {
        return;
    }

    let currently_in_truck = !q_in_truck.is_empty();
    let previously_in_truck = *was_in_truck;
    *was_in_truck = currently_in_truck;

    // Player leaves the truck
    if !currently_in_truck && previously_in_truck {
        *exited_truck_time = Some(time.elapsed_secs_f64());

        // Check if the current player has empty right hand
        if let Some((_player_sprite, player_gear)) = player_gear_q.iter().next() {
            // Calculate current set of discoverable evidences from player's gear
            let mut new_discoverable_evidences = HashSet::new();
            let mut new_has_repellent_flask = false;

            // Check left hand gear
            if let Some(entity) = player_gear.left_hand
                && let Ok(kind) = q_gear.get(entity)
            {
                if let Ok(evidence) = Evidence::try_from(kind) {
                    new_discoverable_evidences.insert(evidence);
                }
                if *kind == GearKind::RepellentFlask {
                    new_has_repellent_flask = true;
                }
            }

            // Check right hand gear
            if let Some(entity) = player_gear.right_hand
                && let Ok(kind) = q_gear.get(entity)
            {
                if let Ok(evidence) = Evidence::try_from(kind) {
                    new_discoverable_evidences.insert(evidence);
                }
                if *kind == GearKind::RepellentFlask {
                    new_has_repellent_flask = true;
                }
            }

            // Check inventory gear
            for &entity in &player_gear.inventory {
                if let Ok(kind) = q_gear.get(entity) {
                    if let Ok(evidence) = Evidence::try_from(kind) {
                        new_discoverable_evidences.insert(evidence);
                    }
                    if *kind == GearKind::RepellentFlask {
                        new_has_repellent_flask = true;
                    }
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
            *empty_right_handed = player_gear.right_hand.is_none();
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
            && crate::triggers::net::walkie_set_or_propose(
                WalkieEvent::PlayerLeavesTruckWithoutChangingLoadout,
                cur_time,
                &mut walkie_play,
                &mut ev_propose,
            )
        {
            *exited_truck_time = None;
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        trigger_all_objectives_met_reminder_system,
    )
    .add_systems(
        Update,
        trigger_player_leaves_truck_without_changing_loadout_system
            ,
    );
}
