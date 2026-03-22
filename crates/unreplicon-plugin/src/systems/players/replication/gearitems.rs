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
        super::super::noop_write::<Flashlight>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, UVTorch>(
        super::super::noop_write::<UVTorch>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, RedTorch>(
        super::super::noop_write::<RedTorch>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, RepellentFlask>(
        super::super::noop_write::<RepellentFlask>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, SaltData>(
        super::super::noop_write::<SaltData>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, SageBundleData>(
        super::super::noop_write::<SageBundleData>,
        super::super::noop_remove,
    );
    app.set_marker_fns::<LocallyOwned, QuartzStoneData>(
        super::super::noop_write::<QuartzStoneData>,
        super::super::noop_remove,
    );
}
