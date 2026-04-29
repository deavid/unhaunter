use crate::assets::TruckUiAssets;
use crate::colors;
use bevy::prelude::*;
use uncommon_app_core::platform::plt::{FONT_SCALE, UI_SCALE};
use uninput_core::states::InGameUiState;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unvitals_core::components::PlayerVitals;

const MARGIN_PERCENT: f32 = 0.5 * UI_SCALE;
const TEXT_MARGIN: UiRect = UiRect::percent(2.0 * UI_SCALE, 0.0, 0.0, 0.0);

#[derive(Component, Debug)]
pub(crate) struct SanityText;

pub(crate) fn setup_sanity_ui(p: &mut ChildSpawnerCommands, handles: &TruckUiAssets) {
    let title = (
        Text::new("Sanity"),
        TextFont {
            font: handles.font_londrina_light.clone(),
            font_size: 35.0 * FONT_SCALE,
            ..default()
        },
        TextColor(colors::TRUCKUI_ACCENT_COLOR),
        Node {
            height: Val::Px(40.0 * UI_SCALE),
            ..default()
        },
    );
    p.spawn(title);

    // Sanity contents
    p.spawn(Node {
        border: UiRect::top(Val::Px(2.0 * UI_SCALE)),
        height: Val::Px(0.0 * UI_SCALE),
        ..default()
    })
    .insert(BorderColor::all(colors::TRUCKUI_ACCENT_COLOR));
    let p1_sanity = (
        Text::new("Loading player data..."),
        TextFont {
            font: handles.font_chakra_light.clone(),
            font_size: 25.0 * FONT_SCALE,
            ..default()
        },
        TextColor(colors::TRUCKUI_TEXT_COLOR),
        Node {
            margin: TEXT_MARGIN,
            ..default()
        },
    );
    p.spawn(p1_sanity).insert(SanityText);
    p.spawn(Node {
        justify_content: JustifyContent::FlexStart,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Percent(MARGIN_PERCENT),
        flex_grow: 1.0,
        ..default()
    });
}

fn update_sanity(
    qp: Query<(&PlayerVitals, &PlayerSprite, Has<MainPlayer>)>,
    mut qst: Query<&mut Text, With<SanityText>>,
) {
    let mut players: Vec<_> = qp.iter().collect();
    players.sort_by(|a, b| {
        // Sort MainPlayer first
        if a.2 != b.2 {
            return b.2.cmp(&a.2);
        }
        // Then by network_id
        a.1.network_id.0.cmp(&b.1.network_id.0)
    });

    let mut lines = Vec::new();
    for (vitals, sprite, _is_main) in players {
        let name = unreplicon_core::identity::generate_deterministic_name(sprite.id);
        lines.push(format!(
            "{}:\n  {:.0}% Sanity, {:.0}% Health",
            name, vitals.sanity, vitals.health
        ));
    }

    let new_text = if lines.is_empty() {
        "No player data available".to_string()
    } else {
        lines.join("\n")
    };

    for mut text in &mut qst {
        if text.0 != new_text {
            text.0 = new_text.clone();
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        update_sanity.run_if(in_state(InGameUiState::Truck)),
    );
}
