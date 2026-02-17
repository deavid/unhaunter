use bevy::prelude::*;
use unnet_core::resources::RoomIdentification;
use untypes_core::cli::CliOptions;
use untypes_core::states::AppState;

use crate::{hub_client, ui};

pub struct UnhaunterHubPlugin;

impl Plugin for UnhaunterHubPlugin {
    fn build(&self, app: &mut App) {
        let cli = app
            .world()
            .get_resource::<CliOptions>()
            .cloned()
            .unwrap_or_default();

        // Unconditionally initialize RoomIdentification
        app.init_resource::<RoomIdentification>();

        if !cli.dedicated {
            // Client side logic
            app.add_systems(
                Startup,
                (hub_client::setup_hub_client, hub_client::initialize_nickname),
            );
            app.add_systems(Update, hub_client::update_hub_status);

            // UI systems
            app.add_systems(OnEnter(AppState::Hub), ui::setup_hub_ui);
            app.add_systems(
                Update,
                (ui::hub_menu_event, ui::update_code_input).run_if(in_state(AppState::Hub)),
            );
            app.add_systems(
                Update,
                ui::handle_hub_responses.run_if(in_state(AppState::Hub)),
            );
            app.add_systems(OnExit(AppState::Hub), ui::despawn_hub_ui);
        }
    }
}
