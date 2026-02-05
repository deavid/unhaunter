use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub(crate) const UPDATE_QUARTZ_AND_GHOST: DP =
    DP::const_new("ungearitems/systems/quartz/update_quartz_and_ghost");
pub(crate) const EMF_UPDATE: DP = DP::const_new("ungearitems/systems/emfmeter/update_emfmeter");
pub(crate) const GEIGER_UPDATE: DP =
    DP::const_new("ungearitems/systems/geigercounter/update_geigercounter");
pub(crate) const SPIRITBOX_UPDATE: DP =
    DP::const_new("ungearitems/systems/spiritbox/update_spiritbox");
pub(crate) const FLASHLIGHT_UPDATE: DP =
    DP::const_new("ungearitems/systems/flashlight/update_flashlight");
pub(crate) const UVTORCH_UPDATE: DP = DP::const_new("ungearitems/systems/uvtorch/update_uvtorch");
pub(crate) const REDTORCH_UPDATE: DP =
    DP::const_new("ungearitems/systems/redtorch/update_redtorch");
pub(crate) const PHOTOCAM_UPDATE: DP =
    DP::const_new("ungearitems/systems/photocam/update_photocam");
pub(crate) const VIDEOCAM_UPDATE: DP =
    DP::const_new("ungearitems/systems/videocam/update_videocam");
pub(crate) const IONMETER_UPDATE: DP =
    DP::const_new("ungearitems/systems/ionmeter/update_ionmeter");
pub(crate) const ESTATICMETER_UPDATE: DP =
    DP::const_new("ungearitems/systems/estaticmeter/update_estaticmeter");
pub(crate) const COMPASS_UPDATE: DP = DP::const_new("ungearitems/systems/compass/update_compass");
pub(crate) const MOTIONSENSOR_UPDATE: DP =
    DP::const_new("ungearitems/systems/motionsensor/update_motionsensor");
pub(crate) const THERMALIMAGER_UPDATE: DP =
    DP::const_new("ungearitems/systems/thermalimager/update_thermalimager");
pub(crate) const SALT_PARTICLE: DP = DP::const_new("ungearitems/systems/salt/salt_particle");
pub(crate) const SALT_PILE: DP = DP::const_new("ungearitems/systems/salt/salt_pile");
pub(crate) const SALTY_TRACE: DP = DP::const_new("ungearitems/systems/salt/salty_trace");
pub(crate) const SAGE_SMOKE: DP = DP::const_new("ungearitems/systems/sage/sage_smoke");
pub(crate) const TEMPERATURE_UPDATE: DP =
    DP::const_new("ungearitems/systems/thermometer/temperature_update");
pub(crate) const SOUND_UPDATE: DP = DP::const_new("ungearitems/systems/recorder/sound_update");
pub(crate) const REPELLENT_UPDATE: DP =
    DP::const_new("ungearitems/systems/repellentflask/repellent_update");
pub(crate) const ELECTRONIC_INTERFERENCE: DP =
    DP::const_new("ungearitems/systems/electronic_interference");
pub(crate) const BATTERY_DRAIN: DP = DP::const_new("ungearitems/systems/battery_drain");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(UPDATE_QUARTZ_AND_GHOST).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(EMF_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GEIGER_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(SPIRITBOX_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(FLASHLIGHT_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(UVTORCH_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(REDTORCH_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(PHOTOCAM_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(VIDEOCAM_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(IONMETER_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(ESTATICMETER_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(COMPASS_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(MOTIONSENSOR_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(THERMALIMAGER_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(SALT_PARTICLE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(SALT_PILE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(SALTY_TRACE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(SAGE_SMOKE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(TEMPERATURE_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(SOUND_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(REPELLENT_UPDATE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(ELECTRONIC_INTERFERENCE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(BATTERY_DRAIN).with_suffix("ms"));
}
