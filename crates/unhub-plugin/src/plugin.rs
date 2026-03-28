use bevy::prelude::*;
use unorchestrator_core::UIContextState;
use unreplicon_core::messages::HubConnectionRequested;
use unreplicon_core::resources::RoomIdentification;

use crate::{hub_client, ui};

pub struct UnhaunterHubCorePlugin;

impl Plugin for UnhaunterHubCorePlugin {
    fn build(&self, app: &mut App) {
        // Unconditionally initialize RoomIdentification
        app.init_resource::<RoomIdentification>();
        // Register the hub connection event that signals to the transport layer
        app.add_message::<HubConnectionRequested>();
    }
}

pub struct UnhaunterHubPlugin {
    pub hub_url: Option<String>,
}

impl Plugin for UnhaunterHubPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(hub_client::HubConfig {
            hub_url: self.hub_url.clone(),
        });

        // Client side logic
        app.add_systems(Startup, hub_client::setup_hub_client);
        app.add_systems(Update, hub_client::update_hub_status);

        // UI systems
        app.add_systems(OnEnter(UIContextState::Hub), ui::setup_hub_ui);
        app.add_systems(
            Update,
            (ui::hub_menu_event, ui::update_code_input).run_if(in_state(UIContextState::Hub)),
        );
        app.add_systems(
            Update,
            ui::handle_hub_responses.run_if(in_state(UIContextState::Hub)),
        );
        app.add_systems(OnExit(UIContextState::Hub), ui::despawn_hub_ui);
    }
}
