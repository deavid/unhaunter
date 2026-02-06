use bevy::diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic};
use bevy::prelude::*;

pub(crate) const TRIGGER_HUNT_ACTIVE_NEAR_HIDING_SPOT_NO_HIDE: DP =
    DP::const_new("unwalkie/systems/trigger_hunt_active_near_hiding_spot_no_hide");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(
        Diagnostic::new(TRIGGER_HUNT_ACTIVE_NEAR_HIDING_SPOT_NO_HIDE).with_suffix("ms"),
    );
}
