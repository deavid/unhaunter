pub(crate) mod visual_effects;

pub(crate) fn app_setup(app: &mut bevy::prelude::App) {
    visual_effects::app_setup(app);
}
