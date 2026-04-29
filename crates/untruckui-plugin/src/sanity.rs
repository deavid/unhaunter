use crate::assets::TruckUiAssets;
use crate::colors;
use bevy::color::palettes::css;
use bevy::prelude::*;
use std::collections::HashMap;
use uncommon_app_core::platform::plt::{FONT_SCALE, UI_SCALE};
use uninput_core::states::InGameUiState;
use unplayer_core::colors::player_color;
use unplayer_core::components::{MainPlayer, PlayerDisconnected, PlayerInactive, PlayerSprite};
use unreplicon_core::components::LobbyInfo;
use untruck_core::components::in_truck::InTruck;
use unvitals_core::components::PlayerVitals;
use uuid::Uuid;

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

    p.spawn((
        Node {
            margin: TEXT_MARGIN,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        SanityText,
    ));

    p.spawn(Node {
        justify_content: JustifyContent::FlexStart,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Percent(MARGIN_PERCENT),
        flex_grow: 1.0,
        ..default()
    });
}

fn update_sanity(
    mut commands: Commands,
    handles: Res<TruckUiAssets>,
    qp: Query<(
        &PlayerVitals,
        &PlayerSprite,
        Has<MainPlayer>,
        Has<PlayerDisconnected>,
        Has<PlayerInactive>,
        Has<InTruck>,
    )>,
    q_lobby: Query<&LobbyInfo>,
    q_container: Query<Entity, With<SanityText>>,
    mut q_text: Query<&mut Text>,
    mut last_order: Local<Vec<(Uuid, bool, bool, bool, bool, u8)>>,
    mut text_map: Local<HashMap<Uuid, Entity>>,
) {
    let Ok(container_entity) = q_container.single() else {
        return;
    };

    let lobby_info = q_lobby.single().ok();

    let mut players: Vec<_> = qp.iter().collect();
    players.sort_by(|a, b| {
        // Sort MainPlayer first
        if a.2 != b.2 {
            return b.2.cmp(&a.2);
        }
        // Then by connection state (Active > Inactive > Disconnected)
        // a.3=disc, a.4=inact. We want disc last, then inact.
        let state_score = |disc, inact| if disc { 2 } else if inact { 1 } else { 0 };
        let score_a = state_score(a.3, a.4);
        let score_b = state_score(b.3, b.4);
        if score_a != score_b {
            return score_a.cmp(&score_b);
        }
        // Then by network_id
        a.1.network_id.0.cmp(&b.1.network_id.0)
    });

    let current_order: Vec<_> = players
        .iter()
        .map(|(_, s, main, disc, inact, in_truck)| {
            let tint = lobby_info
                .and_then(|li| li.players.iter().find(|p| p.player_uuid == s.id))
                .map(|p| p.tint_color_index)
                .unwrap_or(0);
            (s.id, *main, *disc, *inact, *in_truck, tint)
        })
        .collect();

    if *last_order != current_order {
        *last_order = current_order;
        text_map.clear();
        commands.entity(container_entity).despawn_children();

        if players.is_empty() {
            commands.entity(container_entity).with_children(|p| {
                p.spawn((
                    Text::new("No player data available"),
                    TextFont {
                        font: handles.font_chakra_light.clone(),
                        font_size: 18.0 * FONT_SCALE,
                        ..default()
                    },
                    TextColor(colors::TRUCKUI_TEXT_COLOR),
                ));
            });
        } else {
            for (vitals, sprite, _main, is_disc, is_inact, is_in_truck) in &players {
                let tint = lobby_info
                    .and_then(|li| li.players.iter().find(|p| p.player_uuid == sprite.id))
                    .map(|p| p.tint_color_index)
                    .unwrap_or(0);

                let color = if *is_disc {
                    css::GRAY.into()
                } else if *is_inact {
                    css::YELLOW.into()
                } else if *is_in_truck {
                    // Light pale green as requested
                    Color::srgba(0.7, 0.9, 0.7, 1.0)
                } else {
                    colors::TRUCKUI_TEXT_COLOR
                };

                let player_tint = player_color(tint as usize);

                commands.entity(container_entity).with_children(|p| {
                    p.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(5.0 * UI_SCALE),
                        ..default()
                    })
                    .with_children(|row| {
                        // Tint square
                        row.spawn((
                            Node {
                                width: Val::Px(12.0 * UI_SCALE),
                                height: Val::Px(12.0 * UI_SCALE),
                                ..default()
                            },
                            BackgroundColor(player_tint),
                        ));
                        // Status text
                        let name = unreplicon_core::identity::generate_deterministic_name(sprite.id);
                        let status_text = format!(
                            "{}:\n  {:.0}% Sanity, {:.0}% Health",
                            name, vitals.sanity, vitals.health
                        );
                        let text_entity = row
                            .spawn((
                                Text::new(status_text),
                                TextFont {
                                    font: handles.font_chakra_light.clone(),
                                    font_size: 18.0 * FONT_SCALE,
                                    ..default()
                                },
                                TextColor(color),
                            ))
                            .id();
                        text_map.insert(sprite.id, text_entity);
                    });
                });
            }
        }
    }

    // Update text content every frame
    for (vitals, sprite, _, _, _, _) in players {
        if let Some(&text_entity) = text_map.get(&sprite.id) {
            if let Ok(mut text) = q_text.get_mut(text_entity) {
                let name = unreplicon_core::identity::generate_deterministic_name(sprite.id);
                let new_content = format!(
                    "{}:\n  {:.0}% Sanity, {:.0}% Health",
                    name, vitals.sanity, vitals.health
                );
                if text.0 != new_content {
                    text.0 = new_content;
                }
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        update_sanity.run_if(in_state(InGameUiState::Truck)),
    );
}
