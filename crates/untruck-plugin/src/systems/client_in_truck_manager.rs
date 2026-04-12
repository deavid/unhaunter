//! Reactively manages the client's `InGameUiState` based on the local player's physical presence in the truck.
//!
//! This module acts as the bridge between Tier 1 (Gameplay) and Tier 4 (UI). By making the UI
//! observe `InTruck` component insertions/removals rather than mutating state directly via commands,
//! we ensure the UI strictly matches the server-authoritative simulation reality.

use bevy::prelude::*;
use uninput_core::states::InGameUiState;
use unplayer_core::components::MainPlayer;
use untruck_core::components::in_truck::InTruck;

/// Observer that activates the Truck UI when the local player enters the truck.
///
/// We use an observer on the `InTruck` component rather than direct UI state changes
/// so that the presentation layer strictly follows the gameplay reality. The `With<MainPlayer>`
/// filter ensures that replicated `InTruck` components from remote players don't hijack
/// the local client's screen.
fn on_intruck_added(
    trigger: On<Add, InTruck>,
    mut next_state: ResMut<NextState<InGameUiState>>,
    q_main: Query<(), With<MainPlayer>>,
) {
    if q_main.contains(trigger.entity) {
        next_state.set(InGameUiState::Truck);
    }
}

/// Observer that dismisses the Truck UI when the local player exits the truck.
///
/// Checking for `With<MainPlayer>` guarantees we only transition the local UI back
/// to the running state when the local player physically exits the truck.
fn on_intruck_removed(
    trigger: On<Remove, InTruck>,
    mut next_state: ResMut<NextState<InGameUiState>>,
    q_main: Query<(), With<MainPlayer>>,
) {
    if q_main.contains(trigger.entity) {
        next_state.set(InGameUiState::Running);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_observer(on_intruck_added);
    app.add_observer(on_intruck_removed);
}
