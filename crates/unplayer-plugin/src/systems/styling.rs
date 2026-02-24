use bevy::prelude::*;
use unboard_core::components::mapcolor::MapColor;
use unfoundation_core::colors;
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::resources::LobbyData;
use unplayer_core::components::Hiding;
use unplayer_core::components::MainPlayer;
use unplayer_core::components::PlayerSpectating;
use unplayer_core::components::PlayerSprite;
use unreplicon_core::net_components::PlayerNetInfo;

pub(crate) fn update_player_styling(
    lobby_data: Option<Res<LobbyData>>,
    mut query: Query<
        (
            &NetworkId,
            &mut MapColor,
            Has<Hiding>,
            Has<MainPlayer>,
            Has<PlayerSpectating>,
            Option<&PlayerNetInfo>,
        ),
        With<PlayerSprite>,
    >,
) {
    for (id, mut map_color, is_hiding, is_main, is_spectating, maybe_net_info) in query.iter_mut() {
        let tint_index = if let Some(net_info) = maybe_net_info {
            // Replicon mode: match by client_id stored in PlayerNetInfo.
            lobby_data
                .as_ref()
                .and_then(|ld| {
                    ld.players
                        .iter()
                        .find(|p| p.id.0 == net_info.client_id)
                        .map(|p| p.tint_color_index as usize)
                })
                .unwrap_or(net_info.tint_color_index as usize)
        } else {
            // Legacy / offline mode: match by NetworkId.
            lobby_data
                .as_ref()
                .and_then(|ld| {
                    ld.players
                        .iter()
                        .find(|p| p.id == *id)
                        .map(|p| p.tint_color_index as usize)
                })
                .unwrap_or(id.0 as usize % 9)
        };

        let mut color = colors::player_color(tint_index);

        // Make the in-mission tint lighter (20% brighter) so it doesn't overpower the sprite details.
        if let Color::Hsla(mut hsla) = color {
            hsla.lightness = (hsla.lightness * 1.2).min(1.0);
            color = Color::Hsla(hsla);
        }

        let alpha = if is_spectating {
            if is_main { 0.5 } else { 0.0 }
        } else if is_hiding {
            0.2
        } else {
            1.0
        };

        color.set_alpha(alpha);

        if map_color.color != color {
            map_color.color = color;
        }
    }
}
