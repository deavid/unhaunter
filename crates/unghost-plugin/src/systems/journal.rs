use bevy::prelude::*;
use bevy_replicon::prelude::{Channel, ClientMessageAppExt, FromClient};
use unghost_core::events::{JournalEvidenceToggled, JournalGhostToggled};
use uninvestigation_core::messages::{RequestJournalEvidenceToggle, RequestJournalGhostToggle};
use uninvestigation_core::resources::ghost_guess::GhostGuess;
use unmission_core::types::SimulationState;
use unreplicon_core::resources::AuthorityRole;

pub(crate) fn app_setup(app: &mut App) {
    app.add_client_message::<RequestJournalEvidenceToggle>(Channel::Ordered);
    app.add_client_message::<RequestJournalGhostToggle>(Channel::Ordered);
    app.add_systems(
        Update,
        (handle_journal_evidence_toggle, handle_journal_ghost_toggle)
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(SimulationState::Ready)),
    );
    app.add_systems(
        Update,
        (apply_journal_evidence_toggle, apply_journal_ghost_toggle)
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(SimulationState::Ready)),
    );
}

fn handle_journal_evidence_toggle(
    mut reader: MessageReader<FromClient<RequestJournalEvidenceToggle>>,
    mut writer: MessageWriter<JournalEvidenceToggled>,
) {
    for msg in reader.read() {
        info!(
            "JOURNAL_NET: received evidence toggle from client {:?}: evidence={:?} discard={} mark_as_found={}",
            msg.client_id, msg.message.evidence, msg.message.discard, msg.message.mark_as_found
        );
        writer.write(JournalEvidenceToggled {
            evidence: msg.message.evidence,
            discard: msg.message.discard,
            mark_as_found: msg.message.mark_as_found,
        });
    }
}

fn handle_journal_ghost_toggle(
    mut reader: MessageReader<FromClient<RequestJournalGhostToggle>>,
    mut writer: MessageWriter<JournalGhostToggled>,
) {
    for msg in reader.read() {
        info!(
            "JOURNAL_NET: received ghost toggle from client {:?}: ghost={:?} discard={}",
            msg.client_id, msg.message.ghost_type, msg.message.discard
        );
        writer.write(JournalGhostToggled {
            ghost_type: msg.message.ghost_type,
            discard: msg.message.discard,
        });
    }
}

pub(crate) fn apply_journal_evidence_toggle(
    mut reader: MessageReader<JournalEvidenceToggled>,
    mut ghost_guess: Option<ResMut<GhostGuess>>,
) {
    let Some(ref mut ghost_guess) = ghost_guess else {
        warn!("apply_journal_evidence_toggle: GhostGuess resource missing");
        return;
    };
    for msg in reader.read() {
        let before = ghost_guess.clone();
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
        info!(
            "JOURNAL_APPLY: evidence toggle applied evidence={:?} discard={} mark_as_found={} before={:?} after={:?}",
            msg.evidence, msg.discard, msg.mark_as_found, before, *ghost_guess
        );
    }
}

pub(crate) fn apply_journal_ghost_toggle(
    mut reader: MessageReader<JournalGhostToggled>,
    mut ghost_guess: Option<ResMut<GhostGuess>>,
) {
    let Some(ref mut ghost_guess) = ghost_guess else {
        warn!("apply_journal_ghost_toggle: GhostGuess resource missing");
        return;
    };
    for msg in reader.read() {
        let before = ghost_guess.clone();
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
        info!(
            "JOURNAL_APPLY: ghost toggle applied ghost={:?} discard={} before={:?} after={:?}",
            msg.ghost_type, msg.discard, before, *ghost_guess
        );
    }
}
