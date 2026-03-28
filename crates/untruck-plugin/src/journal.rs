use bevy::prelude::*;
use unghost_core::resources::ghost_guess::GhostGuess;
use uninput_core::states::InGameUiState;
use untruck_core::components::truck_ui_button::TruckUIButton;
use untruck_core::components::truck_ui_markers::TruckUIGhostGuess;
use untruck_core::journal::ForceDiscardEvidenceEvent;
use untruck_core::types::truck_button::{TruckButtonState, TruckButtonType};

/// System that handles ForceDiscardEvidenceEvents even when not in truck
fn force_discard_evidence_system(
    mut interaction_query: Query<&mut TruckUIButton, With<Button>>,
    mut ev_force_discard: MessageReader<ForceDiscardEvidenceEvent>,
    mut gg: ResMut<GhostGuess>,
) {
    for event in ev_force_discard.read() {
        debug!(
            "Journal: Received ForceDiscardEvidenceEvent for {:?}",
            event.0
        );

        let mut button_found = false;
        for mut tui_button in interaction_query.iter_mut() {
            if let TruckButtonType::Evidence(evidence_type) = tui_button.class
                && evidence_type == event.0
            {
                debug!(
                    "Journal: Setting evidence {:?} button from {:?} to Discard",
                    evidence_type, tui_button.status
                );
                tui_button.status = TruckButtonState::Discard;
                tui_button.computer_locked = true;
                button_found = true;
                break;
            }
        }

        if button_found {
            // Update the model to reflect the discarded state
            gg.evidences_found.remove(&event.0);
            gg.evidences_missing.insert(event.0);

            // Force mark the GhostGuess as changed to trigger update systems
            gg.set_changed();
            debug!(
                "Journal: ForceDiscardEvidenceEvent processed for {:?}",
                event.0
            );
        } else {
            warn!("Journal: Could not find evidence button for {:?}", event.0);
        }
    }
}

fn ghost_guess_system(gg: Res<GhostGuess>, mut q_gg: Query<&mut Text, With<TruckUIGhostGuess>>) {
    if !gg.is_changed() {
        return;
    }
    let ghost_name = gg
        .ghost_type
        .map(|g| g.name().to_string())
        .unwrap_or_else(|| "-- Unknown --".to_string());

    for mut text in q_gg.iter_mut() {
        if text.0 != ghost_name {
            text.0 = ghost_name.clone();
        }
    }
}

pub(crate) fn app_setup_core(app: &mut App) {
    app.add_message::<ForceDiscardEvidenceEvent>();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        ghost_guess_system.run_if(in_state(InGameUiState::Truck)),
    )
    .add_systems(Update, force_discard_evidence_system);
}
