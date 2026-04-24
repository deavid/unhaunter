use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub(crate) const SALT_PILE: DP = DP::const_new("ungearitems/systems/salt/salt_pile");
pub(crate) const ELECTRONIC_INTERFERENCE: DP =
    DP::const_new("ungearitems/systems/electronic_interference");
pub(crate) const BATTERY_DRAIN: DP = DP::const_new("ungearitems/systems/battery_drain");
pub(crate) const UVTORCH_UPDATE: DP = DP::const_new("ungearitems/systems/uvtorch/update_uvtorch");
pub(crate) const REDTORCH_UPDATE: DP =
    DP::const_new("ungearitems/systems/redtorch/update_redtorch");
pub(crate) const FLASHLIGHT_UPDATE: DP =
    DP::const_new("ungearitems/systems/flashlight/update_flashlight");
pub(crate) const VIDEOCAM_UPDATE: DP =
    DP::const_new("ungearitems/systems/videocam/update_videocam");
pub(crate) const UPDATE_QUARTZ_AND_GHOST: DP =
    DP::const_new("ungearitems/systems/quartz/update_quartz_and_ghost");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(SALT_PILE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(ELECTRONIC_INTERFERENCE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(BATTERY_DRAIN).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(UVTORCH_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(REDTORCH_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(FLASHLIGHT_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(VIDEOCAM_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(UPDATE_QUARTZ_AND_GHOST).with_suffix("ms"));
}
