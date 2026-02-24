use bevy::prelude::*;
use unplayer_core::components::{Hiding, MainPlayer};
use untruck_core::components::in_truck::InTruck;
use untypes_core::states::GameState;

/// System that adds InTruck component when local player enters GameState::Truck
pub(crate) fn on_enter_truck(
    mut commands: Commands,
    query: Query<Entity, (With<MainPlayer>, Without<InTruck>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert(InTruck)
            .insert(Hiding { hiding_spot: None });
    }
}

/// System that removes InTruck component when local player exits GameState::Truck
pub(crate) fn on_exit_truck(
    mut commands: Commands,
    query: Query<Entity, (With<MainPlayer>, With<InTruck>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<InTruck>().remove::<Hiding>();
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(GameState::Truck), on_enter_truck);
    app.add_systems(OnExit(GameState::Truck), on_exit_truck);
}
