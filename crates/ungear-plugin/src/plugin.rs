use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_replicon::prelude::AppMarkerExt;
use bevy_replicon::prelude::AppRuleExt;
use ungear_core::assets::GearAssets;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::{HeldObject, PlayerGear};
use ungear_core::events::{
    RequestEquipGearFromVan, RequestUnequipHand, RequestUnequipInventorySlot,
};
use ungear_core::resources::spawner::{GearMarker, GearSpawnerRegistry};
use ungear_core::types::gear::kind::GearKind;
use unorchestrator_core::UIContextState;
use unreplicon_core::noop::{noop_remove, noop_write};
use unreplicon_core::ownership::LocallyOwned;

use super::systems;
use crate::metrics;

pub struct UnhaunterGearCorePlugin;

impl Plugin for UnhaunterGearCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GearSpawnerRegistry>();

        app.add_message::<RequestEquipGearFromVan>();
        app.add_message::<RequestUnequipHand>();
        app.add_message::<RequestUnequipInventorySlot>();

        app.replicate::<PlayerGear>();
        app.replicate::<HeldObject>();
        app.replicate::<GearMarker>();
        app.replicate::<GearKind>();
        app.replicate::<DeployedGear>();
        app.set_marker_fns::<LocallyOwned, PlayerGear>(noop_write::<PlayerGear>, noop_remove);
        app.set_marker_fns::<LocallyOwned, HeldObject>(noop_write::<HeldObject>, noop_remove);
        app.set_marker_fns::<LocallyOwned, GearMarker>(noop_write::<GearMarker>, noop_remove);
        app.set_marker_fns::<LocallyOwned, GearKind>(noop_write::<GearKind>, noop_remove);
        app.set_marker_fns::<LocallyOwned, DeployedGear>(noop_write::<DeployedGear>, noop_remove);

        crate::net_state::app_setup(app);

        metrics::register_all(app);
    }
}

pub struct UnhaunterGearPlugin;

impl Plugin for UnhaunterGearPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(UIContextState::EngineBoot).load_collection::<GearAssets>(),
        );
        systems::app_setup(app);
    }
}
