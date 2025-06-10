use bevy::prelude::*;
use uncore::colors;
use uncore::components::game_config::GameConfig;
use uncore::components::player_sprite::PlayerSprite;
use uncore::platform::plt::{FONT_SCALE, UI_SCALE};
use uncore::states::GameState;
use uncore::types::root::game_assets::GameAssets;

const MARGIN_PERCENT: f32 = 0.5 * UI_SCALE;
const TEXT_MARGIN: UiRect = UiRect::percent(2.0 * UI_SCALE, 0.0, 0.0, 0.0);

#[derive(Component, Debug)]
pub struct SanityText;

pub fn setup_sanity_ui(p: &mut ChildSpawnerCommands, handles: &GameAssets) {
    let title = (
        Text::new("Sanity"),
        TextFont {
            font: handles.fonts.londrina.w300_light.clone(),
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
    .insert(BorderColor(colors::TRUCKUI_ACCENT_COLOR));
    let p1_sanity = (
        Text::new("Player 1: 90% Sanity"),
        TextFont {
            font: handles.fonts.chakra.w300_light.clone(),
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
    gc: Res<GameConfig>,
    qp: Query<&PlayerSprite>,
    mut qst: Query<&mut Text, With<SanityText>>,
) {
    for player in &qp {
        if player.id != gc.player_id {
            continue;
        }
        for mut text in &mut qst {
            let new_sanity_text = format!("Player 1:\n  {:.0}% Sanity", player.sanity());
            if new_sanity_text != text.0 {
                text.0 = new_sanity_text;
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        update_sanity.run_if(in_state(GameState::Truck)),
    );
}
