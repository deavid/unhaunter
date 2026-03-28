use bevy::prelude::*;
use unghost_core::events::{JournalEvidenceToggled, JournalGhostToggled};
use unghost_core::resources::ghost_guess::GhostGuess;
use untypes_core::roles::AuthorityRole;
use untypes_core::states::SimulationState;

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (apply_journal_evidence_toggle, apply_journal_ghost_toggle)
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(SimulationState::Ready)),
    );
}

pub(crate) fn apply_journal_evidence_toggle(
    mut reader: MessageReader<JournalEvidenceToggled>,
    mut ghost_guess: Option<ResMut<GhostGuess>>,
) {
    let Some(ref mut ghost_guess) = ghost_guess else {
        return;
    };
    for msg in reader.read() {
        if msg.discard {
            if ghost_guess.evidences_missing.contains(&msg.evidence) {
                ghost_guess.evidences_missing.remove(&msg.evidence);
            } else {
                ghost_guess.evidences_missing.insert(msg.evidence);
                ghost_guess.evidences_found.remove(&msg.evidence);
            }
        } else if msg.mark_as_found {
            ghost_guess.evidences_found.insert(msg.evidence);
            ghost_guess.evidences_missing.remove(&msg.evidence);
        } else {
            ghost_guess.evidences_found.remove(&msg.evidence);
            // Non-discard clear maps to "unset".
            ghost_guess.evidences_missing.remove(&msg.evidence);
        }
    }
}

pub(crate) fn apply_journal_ghost_toggle(
    mut reader: MessageReader<JournalGhostToggled>,
    mut ghost_guess: Option<ResMut<GhostGuess>>,
) {
    let Some(ref mut ghost_guess) = ghost_guess else {
        return;
    };
    for msg in reader.read() {
        if msg.discard {
            if let Some(ghost_type) = msg.ghost_type {
                if ghost_guess.ghosts_discarded.contains(&ghost_type) {
                    ghost_guess.ghosts_discarded.remove(&ghost_type);
                } else {
                    ghost_guess.ghosts_discarded.insert(ghost_type);
                    if ghost_guess.ghost_type == Some(ghost_type) {
                        ghost_guess.ghost_type = None;
                    }
                }
            }
        } else {
            ghost_guess.ghost_type = msg.ghost_type;
        }
    }
}
