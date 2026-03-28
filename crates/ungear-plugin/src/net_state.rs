use bevy::prelude::*;
use bevy_replicon::prelude::{Channel, ClientId, ClientMessageAppExt, FromClient};
use ungear_core::components::playergear::PlayerGear;
use unorchestrator_core::UIContextState;
use unplayer_core::components::PlayerSprite;
use unreplicon_core::messages::ExportPlayerGearMessage;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unreplicon_core::resources::{AuthorityRole, LocalPlayerRole};

fn from_owner_id(owner_id: OwnerId) -> ClientId {
    match owner_id {
        OwnerId::Server => ClientId::Server,
        OwnerId::Client(e) => ClientId::Client(e),
    }
}

fn send_export_player_gear(
    q_local: Query<&PlayerGear, (With<LocallyOwned>, With<PlayerSprite>)>,
    mut writer: MessageWriter<ExportPlayerGearMessage>,
) {
    for gear in q_local.iter() {
        writer.write(ExportPlayerGearMessage {
            left_hand: gear.left_hand,
            right_hand: gear.right_hand,
            inventory: gear.inventory.clone(),
            held_item: gear.held_item.clone(),
        });
    }
}

fn handle_export_player_gear_state(
    mut reader: MessageReader<FromClient<ExportPlayerGearMessage>>,
    mut q_players: Query<(&Owner, &mut PlayerGear), Without<LocallyOwned>>,
) {
    // TODO: Theoretical race condition: if an unreliable ExportPlayerGearMessage arrives
    // out-of-order AFTER a RequestDrop has been processed, the server might briefly put
    // the dropped item back into the player's inventory.
    for msg in reader.read() {
        for (owner, mut gear) in q_players.iter_mut() {
            if from_owner_id(owner.0) == msg.client_id {
                gear.left_hand = msg.message.left_hand;
                gear.right_hand = msg.message.right_hand;
                gear.inventory = msg.message.inventory.clone();
                gear.held_item = msg.message.held_item.clone();
                break;
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_mapped_client_message::<ExportPlayerGearMessage>(Channel::Unreliable);
    app.add_systems(
        Update,
        send_export_player_gear
            .run_if(in_state(UIContextState::InGame))
            .run_if(resource_exists::<LocalPlayerRole>),
    );
    app.add_systems(
        Update,
        handle_export_player_gear_state
            .run_if(in_state(UIContextState::InGame))
            .run_if(resource_exists::<AuthorityRole>),
    );
}
