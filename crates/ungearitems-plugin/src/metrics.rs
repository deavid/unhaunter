use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub const UPDATE_QUARTZ_AND_GHOST: DP =
    DP::const_new("ungearitems/systems/quartz/update_quartz_and_ghost");
pub const SALT_PARTICLE: DP = DP::const_new("ungearitems/systems/salt/salt_particle");
pub const SALT_PILE: DP = DP::const_new("ungearitems/systems/salt/salt_pile");
pub const SALTY_TRACE: DP = DP::const_new("ungearitems/systems/salt/salty_trace");
pub const SAGE_SMOKE: DP = DP::const_new("ungearitems/systems/sage/sage_smoke");
pub const TEMPERATURE_UPDATE: DP =
    DP::const_new("ungearitems/systems/thermometer/temperature_update");
pub const SOUND_UPDATE: DP = DP::const_new("ungearitems/systems/recorder/sound_update");
pub const REPELLENT_UPDATE: DP =
    DP::const_new("ungearitems/systems/repellentflask/repellent_update");

pub fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(UPDATE_QUARTZ_AND_GHOST).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(SALT_PARTICLE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(SALT_PILE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(SALTY_TRACE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(SAGE_SMOKE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(TEMPERATURE_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(SOUND_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(REPELLENT_UPDATE).with_suffix("ms"));
}
