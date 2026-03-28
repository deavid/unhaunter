use bevy::prelude::*;
use uncommon_app_core::cli::CliOptions;
use uncommon_app_core::roles::{AuthorityRole, LobbyPresenceRole, LocalPlayerRole};

pub(crate) fn insert_roles_at_startup(cli: Res<CliOptions>, mut commands: Commands) {
    if cli.dedicated {
        // Dedicated server
        commands.insert_resource(AuthorityRole);
        commands.insert_resource(LobbyPresenceRole);
        debug!("Roles inserted: AuthorityRole, LobbyPresenceRole");
    } else {
        match cli.net_mode {
            uncommon_app_core::cli::CliNetMode::Offline => {
                commands.insert_resource(AuthorityRole);
                commands.insert_resource(LocalPlayerRole);
                debug!("Roles inserted: AuthorityRole, LocalPlayerRole");
            }
            uncommon_app_core::cli::CliNetMode::PeerHost { .. } => {
                commands.insert_resource(AuthorityRole);
                commands.insert_resource(LocalPlayerRole);
                commands.insert_resource(LobbyPresenceRole);
                debug!("Roles inserted: AuthorityRole, LocalPlayerRole, LobbyPresenceRole");
            }
            uncommon_app_core::cli::CliNetMode::Join { .. } => {
                commands.insert_resource(LocalPlayerRole);
                commands.insert_resource(LobbyPresenceRole);
                debug!("Roles inserted: LocalPlayerRole, LobbyPresenceRole");
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Startup, insert_roles_at_startup);
}
