use bevy::prelude::*;
use unassets_core::resources::maps::Maps;
use unengine_core::MenuUI;
use unfoundation_core::platform::plt::{FONT_SCALE, UI_SCALE};
use unmenu_core::events::{MenuEscapeEvent, MenuItemClicked, MenuItemSelected};
use unmenu_core::scrollbar::{self, ScrollableListContainer};
use unmenu_core::templates;
use unreplicon_core::messages::RequestSelectMap;
use unreplicon_core::components::LobbyInfo;
use untypes_core::roles::{AuthorityRole, LocalPlayerRole};
use untypes_core::states::LobbyScreen;
use unui_core::assets::UiAssets;

#[derive(Component)]
pub(crate) struct MapSelectUI;

#[derive(Component)]
pub(crate) struct MapSelectPreview;

#[derive(Component)]
pub(crate) struct MapSelectInfo;

#[derive(Resource, Default)]
pub(crate) struct MapSelectMapping {
    pub ui_to_map_index: Vec<usize>,
}

#[derive(Resource, Default)]
pub(crate) struct StateEntryTimer(pub f32);

pub(crate) fn setup_ui(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    maps: Res<Maps>,
    mut mapping: ResMut<MapSelectMapping>,
    time: Res<Time>,
    mut entry_timer: ResMut<StateEntryTimer>,
) {
    *entry_timer = StateEntryTimer(time.elapsed_secs());
    // ... logic continues ...
    mapping.ui_to_map_index = maps.maps.iter().enumerate().map(|(i, _)| i).collect();

    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            MenuUI,
            MapSelectUI,
        ))
        .id();

    commands.entity(root).with_children(|p| {
        templates::create_background(p, &ui_assets);
        templates::create_logo(p, &ui_assets);
        templates::create_breadcrumb_navigation(p, &ui_assets, "Multiplayer Lobby", "Select Map");

        let mut content = templates::create_selectable_content_area(p, &ui_assets, 0);
        content.with_children(|c| {
            // Left: Scrollable List
            c.spawn(Node {
                width: Val::Percent(50.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .with_children(|list_pane| {
                let mut scroll_area = list_pane.spawn((
                    Node {
                        width: Val::Percent(90.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                    ScrollableListContainer,
                ));

                scroll_area.with_children(|sa| {
                    for (ui_idx, &map_idx) in mapping.ui_to_map_index.iter().enumerate() {
                        let map = &maps.maps[map_idx];
                        templates::create_content_item(
                            sa,
                            map.mission_data.display_name.clone(),
                            ui_idx,
                            false,
                            &ui_assets,
                        );
                    }
                });

                // Scrollbar
                list_pane
                    .spawn(Node {
                        width: Val::Percent(10.0),
                        height: Val::Percent(100.0),
                        ..default()
                    })
                    .with_children(|scrollbar_node| {
                        scrollbar::build_scrollbar_ui(scrollbar_node, &ui_assets);
                    });
            });

            // Right: Preview
            c.spawn(Node {
                width: Val::Percent(50.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::left(Val::Px(20.0 * UI_SCALE)),
                row_gap: Val::Px(10.0 * UI_SCALE),
                ..default()
            })
            .with_children(|preview_pane| {
                preview_pane.spawn((
                    ImageNode::default(),
                    Node {
                        width: Val::Percent(100.0),
                        aspect_ratio: Some(16.0 / 9.0),
                        ..default()
                    },
                    MapSelectPreview,
                ));

                preview_pane.spawn((
                    Text::new(""),
                    TextFont {
                        font: ui_assets.font_titillium_regular.clone(),
                        font_size: 20.0 * FONT_SCALE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    MapSelectInfo,
                ));
            });
        });

        templates::create_help_text(
            p,
            &ui_assets,
            Some("[ESC]: Cancel | [Enter/Click]: Confirm Selection".to_string()),
        );
    });
}

pub(crate) fn cleanup_ui(mut commands: Commands, q: Query<Entity, With<MapSelectUI>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

pub(crate) fn handle_input(
    mut ev_clicks: MessageReader<MenuItemClicked>,
    mut ev_escape: MessageReader<MenuEscapeEvent>,
    mut next_lobby_state: ResMut<NextState<LobbyScreen>>,
    q_lobby: Query<&LobbyInfo>,
    mapping: Res<MapSelectMapping>,
    maps: Res<Maps>,
    authority_role: Option<Res<AuthorityRole>>,
    local_player_role: Option<Res<LocalPlayerRole>>,
    time: Res<Time>,
    entry_timer: Res<StateEntryTimer>,
    local_player: Res<unreplicon_core::resources::LocalPlayer>,
    mut ev_send_map: MessageWriter<RequestSelectMap>,
) {
    let lobby_info = q_lobby.single().ok();
    let is_room_owner = match (local_player.0, lobby_info) {
        (Some(lp), Some(li)) => li.leader_uuid == Some(lp),
        (Some(_), None) => authority_role.is_some() && local_player_role.is_some(),
        _ => false,
    };

    // 0.1s guard to avoid "state bounce" from the previous screen's click event
    if time.elapsed_secs() - entry_timer.0 < 0.1 {
        ev_clicks.read().for_each(|_| {}); // Drain events
        ev_escape.read().for_each(|_| {});
        return;
    }

    if ev_escape.read().next().is_some() {
        next_lobby_state.set(LobbyScreen::Main);
    }

    for ev in ev_clicks.read() {
        if !is_room_owner {
            continue;
        }
        if let Some(&map_idx) = mapping.ui_to_map_index.get(ev.pos) {
            let map = &maps.maps[map_idx];
            // Send to server (or echo locally for Host/Offline) via client message.
            // The server handler updates LobbyInfo directly.
            ev_send_map.write(RequestSelectMap {
                map_filepath: map.path.clone(),
            });
            next_lobby_state.set(LobbyScreen::Main);
        }
    }
}

pub(crate) fn update_preview(
    mut ev_selection: MessageReader<MenuItemSelected>,
    mut q_preview: Query<&mut ImageNode, With<MapSelectPreview>>,
    mut q_info: Query<&mut Text, With<MapSelectInfo>>,
    mapping: Res<MapSelectMapping>,
    maps: Res<Maps>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_selection.read() {
        if let Some(&map_idx) = mapping.ui_to_map_index.get(ev.0) {
            let map = &maps.maps[map_idx];

            if let Ok(mut img) = q_preview.single_mut() {
                let path = if map.mission_data.preview_image_path.is_empty() {
                    "img/placeholder_mission.png".to_string()
                } else {
                    map.mission_data.preview_image_path.clone()
                };
                img.image = asset_server.load(path);
            }

            if let Ok(mut text) = q_info.single_mut() {
                text.0 = format!(
                    "{}\n{}\n\n{}",
                    map.mission_data.display_name,
                    map.mission_data.location_name,
                    map.mission_data.flavor_text
                );
            }
        }
    }
}
