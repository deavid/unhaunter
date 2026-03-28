use bevy::prelude::*;
use uninteraction_core::events::{RoomChangedEvent, RoomStateSyncEvent};
use unmission_core::events::LevelReadyEvent;
use unmission_core::types::SimulationState;
use unorchestrator_core::UIContextState;

#[derive(Resource, Default)]
pub(crate) struct PendingMissionStartup {
    open_van: Option<bool>,
}

pub(crate) fn stage_level_ready(
    mut ev_level_ready: MessageReader<LevelReadyEvent>,
    mut pending: ResMut<PendingMissionStartup>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
) {
    let Some(open_van) = ev_level_ready.read().last().map(|event| event.open_van) else {
        return;
    };

    pending.open_van = Some(open_van);
    next_sim_state.set(SimulationState::Spawning);
}

pub(crate) fn enter_in_game_when_ready(
    pending: Res<PendingMissionStartup>,
    mut next_app_state: ResMut<NextState<UIContextState>>,
) {
    if pending.open_van.is_none() {
        return;
    }

    next_app_state.set(UIContextState::InGame);
}

pub(crate) fn sync_initial_room_state_on_enter(
    mut pending: ResMut<PendingMissionStartup>,
    mut ev_room: MessageWriter<RoomChangedEvent>,
    mut ev_room_sync: MessageWriter<RoomStateSyncEvent>,
) {
    let Some(open_van) = pending.open_van.take() else {
        return;
    };

    ev_room_sync.write(RoomStateSyncEvent);
    ev_room.write(RoomChangedEvent::init(open_van));
}

pub(crate) fn reset_pending_mission_startup(mut pending: ResMut<PendingMissionStartup>) {
    pending.open_van = None;
}
