use bevy::prelude::*;
use unmapload_core::components::PendingTiledLayerProperties;
use unmapload_core::events::loadlevel::MapEntitiesReadyEvent;
use untypes_core::hydration::HydrationStage;

use crate::resources::LevelLoadingStatus;

fn hydration_conveyor_system(
    mut commands: Commands,
    q_s4: Query<Entity, With<HydrationStage<4>>>,
    q_s3: Query<Entity, With<HydrationStage<3>>>,
    q_s2: Query<Entity, With<HydrationStage<2>>>,
    q_s1: Query<Entity, With<HydrationStage<1>>>,
    q_any: Query<
        Entity,
        Or<(
            With<HydrationStage<1>>,
            With<HydrationStage<2>>,
            With<HydrationStage<3>>,
            With<HydrationStage<4>>,
        )>,
    >,
    mut loading_status: ResMut<LevelLoadingStatus>,
    mut ev_entities_ready: MessageWriter<MapEntitiesReadyEvent>,
) {
    // Stage 4 Cleanup
    for entity in q_s4.iter() {
        commands
            .entity(entity)
            .remove::<HydrationStage<4>>()
            .remove::<PendingTiledLayerProperties>();
    }

    // Advance Stage 3 to 4
    for entity in q_s3.iter() {
        commands
            .entity(entity)
            .remove::<HydrationStage<3>>()
            .insert(HydrationStage::<4>);
    }

    // Advance Stage 2 to 3
    for entity in q_s2.iter() {
        commands
            .entity(entity)
            .remove::<HydrationStage<2>>()
            .insert(HydrationStage::<3>);
    }

    // Advance Stage 1 to 2
    for entity in q_s1.iter() {
        commands
            .entity(entity)
            .remove::<HydrationStage<1>>()
            .insert(HydrationStage::<2>);
    }

    // Ready Signal
    if *loading_status == LevelLoadingStatus::JustStarted {
        *loading_status = LevelLoadingStatus::InProgress;
    } else if *loading_status == LevelLoadingStatus::InProgress && q_any.is_empty() {
        *loading_status = LevelLoadingStatus::Complete;
        ev_entities_ready.write(MapEntitiesReadyEvent::default());
        info!("Map Hydration Complete");
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(PostUpdate, hydration_conveyor_system);
}
