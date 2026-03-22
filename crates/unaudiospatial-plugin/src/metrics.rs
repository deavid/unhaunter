use bevy::diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic};
use bevy::prelude::*;

pub(crate) const SOUND_PLAYBACK: DP = DP::const_new("unaudiospatial/sound_playback");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(SOUND_PLAYBACK).with_suffix("ms"));
}
