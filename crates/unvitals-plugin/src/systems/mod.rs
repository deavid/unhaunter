pub(crate) mod net_state;
pub(crate) mod other;
pub(crate) mod sanity;
pub(crate) mod setup;
pub(crate) mod spawn;

pub(crate) fn app_setup(app: &mut bevy::prelude::App) {
    setup::app_setup(app);
    spawn::app_setup(app);
    net_state::app_setup(app);
}
