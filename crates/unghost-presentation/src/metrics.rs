use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub(crate) const GHOST_VISUAL_SYNC: DP = DP::const_new("unghost/systems/ghost_visual_sync");
pub(crate) const GIS_SPAWN_PARTICLES: DP =
    DP::const_new("unghost/presentation/gis_spawn_particles");
pub(crate) const GIS_MOTION_BLUR: DP = DP::const_new("unghost/presentation/gis_motion_blur");
pub(crate) const GIS_UPDATE_PARTICLES: DP =
    DP::const_new("unghost/presentation/gis_update_particles");
pub(crate) const GIS_DOOR_LOCK_INDICATOR: DP =
    DP::const_new("unghost/presentation/gis_door_lock_indicator");

pub(crate) fn register_presentation(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(GHOST_VISUAL_SYNC).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GIS_SPAWN_PARTICLES).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GIS_MOTION_BLUR).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GIS_UPDATE_PARTICLES).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GIS_DOOR_LOCK_INDICATOR).with_suffix("ms"));
}
