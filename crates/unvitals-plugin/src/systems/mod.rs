pub(crate) mod other;
pub(crate) mod sanity;
pub(crate) mod setup;

pub(crate) fn app_setup(app: &mut bevy::prelude::App) {
    setup::app_setup(app);
}
