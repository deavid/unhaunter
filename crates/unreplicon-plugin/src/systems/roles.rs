use bevy::prelude::*;
use unreplicon_core::resources::{AuthorityRole, LobbyPresenceRole, LocalPlayerRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetworkRoleIntent {
    #[default]
    Standalone,
    Host,
    Client,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct RoleConfig {
    pub dedicated: bool,
    pub intent: NetworkRoleIntent,
}

pub(crate) fn insert_roles_at_startup(role_config: Res<RoleConfig>, mut commands: Commands) {
    if role_config.dedicated {
        // Dedicated server
        commands.insert_resource(AuthorityRole);
        commands.insert_resource(LobbyPresenceRole);
        debug!("Roles inserted: AuthorityRole, LobbyPresenceRole");
    } else {
        match role_config.intent {
            NetworkRoleIntent::Standalone => {
                commands.insert_resource(AuthorityRole);
                commands.insert_resource(LocalPlayerRole);
                debug!("Roles inserted: AuthorityRole, LocalPlayerRole");
            }
            NetworkRoleIntent::Host => {
                commands.insert_resource(AuthorityRole);
                commands.insert_resource(LocalPlayerRole);
                commands.insert_resource(LobbyPresenceRole);
                debug!("Roles inserted: AuthorityRole, LocalPlayerRole, LobbyPresenceRole");
            }
            NetworkRoleIntent::Client => {
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
