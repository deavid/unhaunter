use bevy::diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic};
use bevy::prelude::*;

pub(crate) const SOUND_UPDATE: DP = DP::const_new("unsound/sound_update");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(SOUND_UPDATE).with_suffix("ms"));
}
