use bevy::prelude::*;
use uncommon_app_core::platform::plt::{FONT_SCALE, UI_SCALE};
use undifficulty_core::difficulty::Difficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unlobby_core::states::LobbyScreen;
use unmenu_core::assets::MenuAssets;
use unmenu_core::components::MenuUI;
use unmenu_core::events::{MenuEscapeEvent, MenuItemClicked, MenuItemSelected};
use unmenu_core::templates;
use unreplicon_core::components::LobbyInfo;
use unreplicon_core::messages::RequestSelectDifficulty;
use unreplicon_core::resources::{AuthorityRole, LocalPlayerRole};

#[derive(Component)]
pub(crate) struct DifficultySelectUI;

#[derive(Component)]
pub(crate) struct DifficultySelectInfo;

#[derive(Resource, Default)]
pub(crate) struct DifficultyMapping {
    pub difficulties: Vec<Difficulty>,
}

#[derive(Resource, Default)]
pub(crate) struct StateEntryTimer(pub f32);

pub(crate) fn setup_ui(
    mut commands: Commands,
    menu_assets: Res<MenuAssets>,
    mut mapping: ResMut<DifficultyMapping>,
    time: Res<Time>,
    mut entry_timer: ResMut<StateEntryTimer>,
) {
    *entry_timer = StateEntryTimer(time.elapsed_secs());
    mapping.difficulties = enum_iterator::all::<Difficulty>().collect();

    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            MenuUI,
            DifficultySelectUI,
        ))
        .id();

    commands.entity(root).with_children(|p| {
        templates::create_background(p, &menu_assets);
        templates::create_logo(p, &menu_assets);
        templates::create_breadcrumb_navigation(
            p,
            &menu_assets,
            "Multiplayer Lobby",
            "Select Difficulty",
        );

        let mut content = templates::create_selectable_content_area(p, &menu_assets, 0);
        content.with_children(|c| {
            // Left: List
            c.spawn(Node {
                width: Val::Percent(40.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|list_pane| {
                for (idx, diff) in mapping.difficulties.iter().enumerate() {
                    templates::create_content_item(
                        list_pane,
                        diff.difficulty_name(),
                        idx,
                        false,
                        &menu_assets,
                    );
                }
            });

            // Right: Description
            c.spawn(Node {
                width: Val::Percent(60.0),
                height: Val::Percent(100.0),
                padding: UiRect::left(Val::Px(20.0 * UI_SCALE)),
                ..default()
            })
            .with_children(|info_pane| {
                info_pane.spawn((
                    Text::new(""),
                    TextFont {
                        font: menu_assets.font_titillium_regular.clone(),
                        font_size: 24.0 * FONT_SCALE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    DifficultySelectInfo,
                ));
            });
        });

        templates::create_help_text(
            p,
            &menu_assets,
            Some("[ESC]: Cancel | [Enter/Click]: Confirm".to_string()),
        );
    });
}

pub(crate) fn cleanup_ui(mut commands: Commands, q: Query<Entity, With<DifficultySelectUI>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

pub(crate) fn handle_input(
    mut ev_clicks: MessageReader<MenuItemClicked>,
    mut ev_escape: MessageReader<MenuEscapeEvent>,
    mut next_lobby_state: ResMut<NextState<LobbyScreen>>,
    q_lobby: Query<&LobbyInfo>,
    mapping: Res<DifficultyMapping>,
    authority_role: Option<Res<AuthorityRole>>,
    local_player_role: Option<Res<LocalPlayerRole>>,
    time: Res<Time>,
    entry_timer: Res<StateEntryTimer>,
    local_player: Res<unreplicon_core::resources::LocalPlayer>,
    mut ev_send_diff: MessageWriter<RequestSelectDifficulty>,
) {
    let lobby_info = q_lobby.single().ok();
    let is_room_owner = match (local_player.uuid, lobby_info) {
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
        if let Some(diff) = mapping.difficulties.get(ev.pos) {
            // Send to server (or echo locally for Host/Offline) via client message.
            // The server handler updates LobbyInfo directly.
            ev_send_diff.write(RequestSelectDifficulty {
                difficulty_id: diff.to_string(),
            });
            next_lobby_state.set(LobbyScreen::Main);
        }
    }
}

pub(crate) fn update_description(
    mut ev_selection: MessageReader<MenuItemSelected>,
    mut q_info: Query<&mut Text, With<DifficultySelectInfo>>,
    mapping: Res<DifficultyMapping>,
) {
    for ev in ev_selection.read() {
        if let Some(diff) = mapping.difficulties.get(ev.0)
            && let Ok(mut text) = q_info.single_mut()
        {
            text.0 = format!(
                "{}\n\n{}",
                diff.difficulty_name(),
                diff.difficulty_description()
            );
        }
    }
}
