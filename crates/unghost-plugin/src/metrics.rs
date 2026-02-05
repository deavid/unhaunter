use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub(crate) const GHOST_MOVEMENT: DP = DP::const_new("unghost/systems/ghost_movement");
pub(crate) const GHOST_ENRAGE: DP = DP::const_new("unghost/systems/ghost_enrage");
pub(crate) const GHOST_BEHAVIOR_DYNAMICS: DP =
    DP::const_new("unghost/systems/ghost_behavior_dynamics");
pub(crate) const GHOST_EMITTER_SYNC: DP = DP::const_new("unghost/systems/ghost_emitter_sync");
pub(crate) const EVIDENCE_DECAY: DP = DP::const_new("unghost/systems/evidence_decay");
pub(crate) const HYDRATION_GHOST_LOGIC: DP = DP::const_new("unghost/systems/hydration_ghost_logic");
pub(crate) const GHOST_VISUAL_SYNC: DP = DP::const_new("unghost/systems/ghost_visual_sync");
pub(crate) const GHOST_INFLUENCE_VISUAL_SYNC: DP =
    DP::const_new("unghost/systems/ghost_influence_visual_sync");
pub(crate) const GIS_TWEEN_ANIMATION: DP = DP::const_new("unghost/systems/gis_tween_animation");
pub(crate) const GIS_DOOR_LOCK_TIMER: DP = DP::const_new("unghost/systems/gis_door_lock_timer");
pub(crate) const GIS_SELECTION: DP = DP::const_new("unghost/systems/gis_selection");
pub(crate) const GIS_EXECUTION: DP = DP::const_new("unghost/systems/gis_execution");
pub(crate) const GIS_SPAWN_PARTICLES: DP = DP::const_new("unghost/systems/gis_spawn_particles");
pub(crate) const GIS_MOTION_BLUR: DP = DP::const_new("unghost/systems/gis_motion_blur");
pub(crate) const GIS_UPDATE_PARTICLES: DP = DP::const_new("unghost/systems/gis_update_particles");
pub(crate) const GIS_DOOR_LOCK_INDICATOR: DP =
    DP::const_new("unghost/systems/gis_door_lock_indicator");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(GHOST_MOVEMENT).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GHOST_ENRAGE).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GHOST_BEHAVIOR_DYNAMICS).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GHOST_EMITTER_SYNC).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(EVIDENCE_DECAY).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(HYDRATION_GHOST_LOGIC).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GHOST_VISUAL_SYNC).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GHOST_INFLUENCE_VISUAL_SYNC).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GIS_TWEEN_ANIMATION).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GIS_DOOR_LOCK_TIMER).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GIS_SELECTION).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GIS_EXECUTION).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GIS_SPAWN_PARTICLES).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GIS_MOTION_BLUR).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GIS_UPDATE_PARTICLES).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(GIS_DOOR_LOCK_INDICATOR).with_suffix("ms"));
}
