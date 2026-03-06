use bevy::prelude::*;

/// Inserted if this process runs authoritative server logic
/// (Dedicated Server or the server-half of PeerHost).
#[derive(Resource, Debug, Default)]
pub struct AuthorityRole;

/// Inserted if this process has a human watching a screen
/// (pure Client or the client-half of PeerHost).
#[derive(Resource, Debug, Default)]
pub struct LocalPlayerRole;

/// Inserted if this session is part of a multiplayer lobby flow
/// (Client, Dedicated Server, or PeerHost — not Offline single-player).
#[derive(Resource, Debug, Default)]
pub struct LobbyPresenceRole;

pub fn is_pure_client(
    local: Option<Res<LocalPlayerRole>>,
    authority: Option<Res<AuthorityRole>>,
) -> bool {
    local.is_some() && authority.is_none()
}

/// Sent by the UI when the local player wants to disconnect from the current
/// multiplayer session and return to single-player offline authority.
///
/// Consumed exclusively by `unreplicon-plugin/src/systems/connection.rs`.
/// The UI must not directly manipulate transport resources; it only writes this message.
#[derive(bevy::prelude::Message, Debug, Clone)]
pub struct DisconnectRequest;
