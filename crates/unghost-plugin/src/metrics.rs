use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub(crate) const GHOST_MOVEMENT: DP = DP::const_new("unghost/systems/ghost_movement");
pub(crate) const GHOST_ENRAGE: DP = DP::const_new("unghost/systems/ghost_enrage");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(GHOST_MOVEMENT).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GHOST_ENRAGE).with_suffix("ms"));
}
