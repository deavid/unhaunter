use bevy::prelude::*;
use unboard_core::components::mapcolor::MapColor;
use unfoundation_core::colors;
use unplayer_core::components::Hiding;
use unplayer_core::components::MainPlayer;
use unplayer_core::components::PlayerSpectating;
use unplayer_core::components::PlayerSprite;
use unreplicon_core::components::LobbyInfo;

pub(crate) fn apply_player_tint_color(
    q_lobby: Query<&LobbyInfo>,
    mut query: Query<(
        &PlayerSprite,
        &mut MapColor,
        Has<Hiding>,
        Has<MainPlayer>,
        Has<PlayerSpectating>,
    )>,
) {
    let lobby_info = q_lobby.single().ok();
    for (ps, mut map_color, is_hiding, is_main, is_spectating) in query.iter_mut() {
        let fallback_tint = ps.network_id.0 as usize % 9;
        let tint_index = lobby_info
            .and_then(|li| {
                li.players
                    .iter()
                    .find(|p| p.player_uuid == ps.id)
                    .map(|p| p.tint_color_index as usize)
            })
            .unwrap_or(fallback_tint);

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
