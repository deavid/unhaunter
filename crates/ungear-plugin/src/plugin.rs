use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::{HeldObject, PlayerGear};
use ungear_core::events::{
    RequestEquipGearFromVan, RequestUnequipHand, RequestUnequipInventorySlot,
};
use ungear_core::resources::spawner::{GearMarker, GearSpawnerRegistry};
use ungear_core::types::gear::kind::GearKind;
use untypes_core::states::AppState;

use super::systems;
use crate::metrics;
use unrender_std::assets::GearAssets;

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

        metrics::register_all(app);
    }
}

pub struct UnhaunterGearPlugin;

impl Plugin for UnhaunterGearPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::EngineBoot).load_collection::<GearAssets>(),
        );
        systems::app_setup(app);
        crate::net_state::app_setup(app);
    }
}
