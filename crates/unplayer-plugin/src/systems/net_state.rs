use bevy::prelude::*;
use bevy_replicon::prelude::{AppMarkerExt, Channel, ClientId, ClientMessageAppExt, FromClient};
use uncommon_states_core::UIContextState;
use unplayer_core::components::{Hiding, PlayerSpectating, PlayerSprite};
use unreplicon_core::messages::ExportPlayerMarkersMessage;
use unreplicon_core::noop::{noop_remove, noop_write};
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unreplicon_core::resources::{AuthorityRole, LocalPlayerRole};
use untruck_core::components::in_truck::InTruck;

fn from_owner_id(owner_id: OwnerId) -> ClientId {
    match owner_id {
        OwnerId::Server => ClientId::Server,
        OwnerId::Client(e) => ClientId::Client(e),
    }
}

fn send_export_player_markers(
    q_local: Query<
        (Has<Hiding>, Has<InTruck>, Has<PlayerSpectating>),
        (With<LocallyOwned>, With<PlayerSprite>),
    >,
    mut writer: MessageWriter<ExportPlayerMarkersMessage>,
) {
    for (is_hiding, in_truck, is_spectating) in q_local.iter() {
        writer.write(ExportPlayerMarkersMessage {
            is_hiding,
            in_truck,
            is_spectating,
        });
    }
}

fn handle_import_player_markers(
    mut reader: MessageReader<FromClient<ExportPlayerMarkersMessage>>,
    mut q_players: Query<
        (Entity, &Owner, Option<&PlayerSpectating>),
        (Without<LocallyOwned>, With<PlayerSprite>),
    >,
    mut commands: Commands,
) {
    for msg in reader.read() {
        for (entity, owner, spectating) in q_players.iter_mut() {
            if from_owner_id(owner.0) != msg.client_id {
                continue;
            }

            if msg.message.is_hiding {
                commands.entity(entity).insert(Hiding { hiding_spot: None });
            } else {
                commands.entity(entity).remove::<Hiding>();
            }

            if msg.message.in_truck {
                commands.entity(entity).insert(InTruck);
            } else {
                commands.entity(entity).remove::<InTruck>();
            }

            if msg.message.is_spectating
                && spectating.is_none()
                && let OwnerId::Client(e) = owner.0
            {
                commands.entity(e).insert(PlayerSpectating);
            }

            break;
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_client_message::<ExportPlayerMarkersMessage>(Channel::Unreliable);
    app.set_marker_fns::<LocallyOwned, Hiding>(noop_write::<Hiding>, noop_remove);
    app.set_marker_fns::<LocallyOwned, InTruck>(noop_write::<InTruck>, noop_remove);
    app.set_marker_fns::<LocallyOwned, PlayerSpectating>(
        noop_write::<PlayerSpectating>,
        noop_remove,
    );
    app.add_systems(
        Update,
        send_export_player_markers
            .run_if(in_state(UIContextState::InGame))
            .run_if(resource_exists::<LocalPlayerRole>),
    );
    app.add_systems(
        Update,
        handle_import_player_markers
            .run_if(in_state(UIContextState::InGame))
            .run_if(resource_exists::<AuthorityRole>),
    );
}
