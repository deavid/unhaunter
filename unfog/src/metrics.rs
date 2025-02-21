use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub const SPAWN_MIASMA: DP = DP::const_new("unfog/systems/spawn_miasma");
pub const ANIMATE_MIASMA: DP = DP::const_new("unfog/systems/animate_miasma");
pub const UPDATE_MIASMA: DP = DP::const_new("unfog/systems/update_miasma");

pub fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(SPAWN_MIASMA).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(ANIMATE_MIASMA).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(UPDATE_MIASMA).with_suffix("ms"));
}
