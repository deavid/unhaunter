pub(crate) mod ghost_dying;
pub(crate) mod ghost_hydration;
pub(crate) mod ghost_orb;
pub(crate) mod ghost_scale_glitch;
pub(crate) mod ghost_traces;
pub(crate) mod gis;
pub(crate) mod visual_sync;

pub(crate) fn app_setup(app: &mut bevy::prelude::App) {
    ghost_dying::app_setup(app);
    ghost_hydration::app_setup(app);
    ghost_orb::app_setup(app);
    ghost_scale_glitch::app_setup(app);
    ghost_traces::app_setup(app);
    gis::app_setup(app);
}
