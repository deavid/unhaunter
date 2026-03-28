use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub(crate) const ANIMATE_SPRITE: DP = DP::const_new("unrender/systems/animate_sprite");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(ANIMATE_SPRITE).with_suffix("ms"));
}
