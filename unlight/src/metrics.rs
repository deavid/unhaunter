use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub const COMPUTE_VISIBILITY: DP = DP::const_new("unlight/functions/compute_visibility");
pub const PLAYER_VISIBILITY: DP = DP::const_new("unlight/systems/player_visibility");
pub const APPLY_LIGHTING: DP = DP::const_new("unlight/systems/apply_lighting");
pub const MARK_FOR_UPDATE: DP = DP::const_new("unlight/systems/mark_for_update");
pub const AMBIENT_SOUND_SYSTEM: DP = DP::const_new("unlight/systems/ambient_sound_system");

pub fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(COMPUTE_VISIBILITY).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(PLAYER_VISIBILITY).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(APPLY_LIGHTING).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(MARK_FOR_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(AMBIENT_SOUND_SYSTEM).with_suffix("ms"));
}
