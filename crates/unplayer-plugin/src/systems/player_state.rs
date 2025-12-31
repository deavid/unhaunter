//! System to update the shared PlayerState resource.

use bevy::prelude::*;
use unplayer_core::resources::PlayerState;
use unspatial_core::position::Position;

use crate::components::player::Hiding;
use crate::components::player_sprite::PlayerSprite;

/// Updates the shared PlayerState resource with current player data.
pub fn update_player_state(
    mut player_state: ResMut<PlayerState>,
    player_query: Query<(&PlayerSprite, &Position, Option<&Hiding>)>,
) {
    if let Ok((player, pos, hiding)) = player_query.single() {
        player_state.id = player.id;
        player_state.health = player.health;
        player_state.sanity = player.sanity();
        player_state.position = *pos;
        player_state.mean_sound = player.mean_sound;
        player_state.controls = player.controls;
        player_state.hiding_spot = hiding.map(|h| h.hiding_spot);
    }
}
