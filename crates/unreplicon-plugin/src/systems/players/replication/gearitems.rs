use bevy::prelude::*;
use bevy_replicon::prelude::AppMarkerExt;
use unreplicon_core::ownership::LocallyOwned;

type Flashlight = ungearitems_core::components::flashlight::Flashlight;
type UVTorch = ungearitems_core::components::uvtorch::UVTorch;
type RedTorch = ungearitems_core::components::redtorch::RedTorch;
type RepellentFlask = ungearitems_core::components::repellentflask::RepellentFlask;
type SaltData = ungearitems_core::components::salt::SaltData;
type SageBundleData = ungearitems_core::components::sage::SageBundleData;
type QuartzStoneData = ungearitems_core::components::quartz::QuartzStoneData;

pub(super) fn register_locally_owned_marker(app: &mut App) {
    app.set_marker_fns::<LocallyOwned, Flashlight>(
        unreplicon_core::noop::noop_write::<Flashlight>,
        unreplicon_core::noop::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, UVTorch>(
        unreplicon_core::noop::noop_write::<UVTorch>,
        unreplicon_core::noop::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, RedTorch>(
        unreplicon_core::noop::noop_write::<RedTorch>,
        unreplicon_core::noop::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, RepellentFlask>(
        unreplicon_core::noop::noop_write::<RepellentFlask>,
        unreplicon_core::noop::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, SaltData>(
        unreplicon_core::noop::noop_write::<SaltData>,
        unreplicon_core::noop::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, SageBundleData>(
        unreplicon_core::noop::noop_write::<SageBundleData>,
        unreplicon_core::noop::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, QuartzStoneData>(
        unreplicon_core::noop::noop_write::<QuartzStoneData>,
        unreplicon_core::noop::noop_remove,
    );
}
