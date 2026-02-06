use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub(crate) const COMPUTE_VISIBILITY: DP = DP::const_new("unlight/functions/compute_visibility");
pub(crate) const PLAYER_VISIBILITY: DP = DP::const_new("unlight/systems/player_visibility");
pub(crate) const PLAYER_EXPOSURE: DP = DP::const_new("unlight/systems/player_exposure");
pub(crate) const APPLY_LIGHTING: DP = DP::const_new("unlight/systems/apply_lighting");
pub(crate) const APPLY_LIGHTING_SPRITES: DP =
    DP::const_new("unlight/systems/apply_lighting_sprites");
pub(crate) const APPLY_MIASMA_CLOUD_VISUALS: DP =
    DP::const_new("unlight/functions/apply_miasma_cloud_visuals");
pub(crate) const AMBIENT_SOUND_SYSTEM: DP = DP::const_new("unlight/systems/ambient_sound_system");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(COMPUTE_VISIBILITY).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(PLAYER_VISIBILITY).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(PLAYER_EXPOSURE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(APPLY_LIGHTING).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(APPLY_LIGHTING_SPRITES).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(APPLY_MIASMA_CLOUD_VISUALS).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(AMBIENT_SOUND_SYSTEM).with_suffix("ms"));
}
