use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub(crate) const SYNC_MAP_ENTITY_FIELD: DP =
    DP::const_new("unrender/systems/sync_map_entity_field");
pub(crate) const ANIMATE_SPRITE: DP = DP::const_new("unrender/systems/animate_sprite");
pub(crate) const HYDRATION_SIMULATION: DP = DP::const_new("unrender/systems/hydration_simulation");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(SYNC_MAP_ENTITY_FIELD).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(ANIMATE_SPRITE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(HYDRATION_SIMULATION).with_suffix("ms"));
}
