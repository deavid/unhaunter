use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub(crate) const APPLY_LIGHTING: DP = DP::const_new("unlight/systems/apply_lighting");
pub(crate) const APPLY_LIGHTING_SPRITES: DP =
    DP::const_new("unlight/systems/apply_lighting_sprites");
pub(crate) const APPLY_MIASMA_CLOUD_VISUALS: DP =
    DP::const_new("unlight/functions/apply_miasma_cloud_visuals");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(APPLY_LIGHTING).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(APPLY_LIGHTING_SPRITES).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(APPLY_MIASMA_CLOUD_VISUALS).with_suffix("ms"));
}
