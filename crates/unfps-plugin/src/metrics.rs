use bevy::diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic};
use bevy::prelude::*;

pub(crate) const LIMIT_REMAINING: DP = DP::const_new("unfps/limit_remaining");
pub(crate) const LIMIT_USAGE: DP = DP::const_new("unfps/limit_usage");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(LIMIT_REMAINING).with_suffix("ms"));
    app.register_diagnostic(Diagnostic::new(LIMIT_USAGE).with_suffix("%"));
}
