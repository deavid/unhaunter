use bevy::prelude::*;
use uncore::behavior::component::Stairs;
use uncore::behavior::{Behavior, Orientation};
use uncore::components::board::position::Position;
use uncore::components::game_config::GameConfig;
use uncore::components::player_sprite::PlayerSprite;

pub fn stairs_player(
    mut players: Query<(&mut Position, &PlayerSprite)>,
    stairs: Query<(&Position, &Stairs, &Behavior), Without<PlayerSprite>>,
    gc: Res<GameConfig>,
) {
    let Some((mut player_pos, _player_sprite)) =
        players.iter_mut().find(|(_, ps)| ps.id == gc.player_id)
    else {
        return;
    };
    let player_bpos = player_pos.to_board_position();
    let mut in_stairs = false;

    for (stair_pos, stair, b) in &stairs {
        let stair_bpos = stair_pos.to_board_position();
        match b.orientation() {
            Orientation::XAxis => {
                if (stair_bpos.x == player_bpos.x || stair_bpos.x + 1 == player_bpos.x)
                    && (player_bpos.y - stair_bpos.y).abs() <= 1
                    && stair_bpos.z == player_bpos.z
                {
                    let dy = player_pos.y - stair_pos.y;
                    player_pos.z = stair_pos.z + (stair.z as f32) / 4.1 + dy / 4.1;
                    if stair.z > 0 {
                        player_pos.z = player_pos.z.clamp(stair_pos.z, stair_pos.z + 1.0);
                    } else {
                        player_pos.z = player_pos.z.clamp(stair_pos.z - 1.0, stair_pos.z);
                    }
                    in_stairs = true;
                }
            }
            Orientation::YAxis => {
                if (stair_bpos.y == player_bpos.y || stair_bpos.y - 1 == player_bpos.y)
                    && (player_bpos.x - stair_bpos.x).abs() <= 1
                    && stair_bpos.z == player_bpos.z
                {
                    let dx = stair_pos.x - player_pos.x;
                    player_pos.z = stair_pos.z + (stair.z as f32) / 4.1 + dx / 4.1;
                    // FIXME: We need to support mirroring the sprite in X direction, meaning that the player would move in the Y direction instead of X.
                    if stair.z > 0 {
                        player_pos.z = player_pos.z.clamp(stair_pos.z, stair_pos.z + 1.0);
                    } else {
                        player_pos.z = player_pos.z.clamp(stair_pos.z - 1.0, stair_pos.z);
                    }
                    in_stairs = true;
                }
            }
            _ => {}
        }
    }

    if !in_stairs && player_pos.z.fract().abs() > 0.001 {
        player_pos.z = player_bpos.z as f32;
    }
}
