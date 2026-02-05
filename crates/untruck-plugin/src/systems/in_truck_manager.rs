use bevy::prelude::*;
use unnet_core::messages::{NetworkDataEvent, NetworkMessage};
use unnet_core::network_id::NetworkId;
use unplayer_core::components::{Hiding, MainPlayer};
use untruck_core::components::in_truck::InTruck;
use untypes_core::cli::{CliOptions, is_host};
use untypes_core::states::GameState;

/// System that adds InTruck component when local player enters GameState::Truck
fn on_enter_truck(
    mut commands: Commands,
    query: Query<(Entity, &NetworkId), (With<MainPlayer>, Without<InTruck>)>,
    cli: Res<CliOptions>,
    mut ev_net: MessageWriter<NetworkDataEvent>,
) {
    let v_host = is_host(cli);
    for (entity, id) in query.iter() {
        commands
            .entity(entity)
            .insert(InTruck)
            .insert(Hiding { hiding_spot: None });

        // If client, notify host (Phase E.2 repurposed RequestTruckEntry)
        if !v_host {
            ev_net.write(NetworkDataEvent {
                message: NetworkMessage::RequestTruckEntry { player_id: *id },
            });
        }
    }
}

/// System that removes InTruck component when local player exits GameState::Truck
fn on_exit_truck(
    mut commands: Commands,
    query: Query<(Entity, &NetworkId), (With<MainPlayer>, With<InTruck>)>,
    cli: Res<CliOptions>,
    mut ev_net: MessageWriter<NetworkDataEvent>,
) {
    let v_host = is_host(cli);
    for (entity, id) in query.iter() {
        commands
            .entity(entity)
            .remove::<InTruck>()
            .remove::<Hiding>();

        // If client, notify host to remove InTruck from Host's entity
        if !v_host {
            ev_net.write(NetworkDataEvent {
                message: NetworkMessage::RequestTruckExit { player_id: *id },
            });
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(GameState::Truck), on_enter_truck);
    app.add_systems(OnExit(GameState::Truck), on_exit_truck);
}
