use bevy::diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic};
use bevy::prelude::*;

pub(crate) const TEMPERATURE_UPDATE: DP = DP::const_new("unthermal/temperature_update");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(TEMPERATURE_UPDATE).with_suffix("ms"));
}
