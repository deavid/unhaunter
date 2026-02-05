use bevy::prelude::*;
use unboard_core::components::mapcolor::MapColor;
use unnet_core::network_id::NetworkId;
use unplayer_core::components::Hiding;
use unplayer_core::components::PlayerSprite;

pub(crate) fn update_player_styling(
    mut query: Query<(&NetworkId, &mut MapColor, Has<Hiding>), With<PlayerSprite>>,
) {
    for (id, mut map_color, is_hiding) in query.iter_mut() {
        let mut color = match id.0 {
            0 => Color::from(bevy::color::palettes::tailwind::GREEN_400),
            1 => Color::from(bevy::color::palettes::tailwind::YELLOW_400),
            2 => Color::from(bevy::color::palettes::tailwind::BLUE_400),
            3 => Color::from(bevy::color::palettes::tailwind::PURPLE_400),
            _ => Color::from(bevy::color::palettes::tailwind::ORANGE_400),
        };

        if is_hiding {
            color.set_alpha(0.5);
        } else {
            color.set_alpha(1.0);
        }

        if map_color.color != color {
            map_color.color = color;
        }
    }
}
