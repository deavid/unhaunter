use crate::hub_client::{ClientProtocolHash, HubClient, HubResponse, HubStatus};
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use uncommon_app_core::platform::plt;
use uncommon_states_core::UIContextState;
use unmenu_core::assets::MenuAssets;
use unmenu_core::components::{MCamera, MenuItemInteractive, MenuRoot, MenuUI};
use unmenu_core::events::{MenuEscapeEvent, MenuItemClicked};
use unmenu_core::templates;
use unreplicon_core::components::LobbyInfo;
use unreplicon_core::messages::HubConnectionRequested;
use unreplicon_core::resources::RoomIdentification;

#[derive(Resource, Default)]
pub struct RoomCodeInput(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum HubMenuID {
    CreateRoom,
    JoinRoom,
    Back,
}

#[derive(Component, Clone, Copy)]
pub struct HubMenuMarker;

#[derive(Component)]
pub struct HubCodeDisplay;

#[derive(Component)]
pub struct HubCodeText;

#[derive(Component)]
pub struct HubStatusLabel;

impl std::fmt::Display for HubMenuID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            HubMenuID::CreateRoom => "Create Room",
            HubMenuID::JoinRoom => "Join Room",
            HubMenuID::Back => "Back",
        };
        f.write_str(text)
    }
}

pub fn setup_hub_ui(mut commands: Commands, ui_assets: Res<MenuAssets>) {
    commands.spawn(Camera2d).insert(MCamera);
    commands.insert_resource(RoomCodeInput::default());

    let menu_items = vec![
        (HubMenuID::CreateRoom, HubMenuID::CreateRoom.to_string()),
        (HubMenuID::JoinRoom, HubMenuID::JoinRoom.to_string()),
        (HubMenuID::Back, HubMenuID::Back.to_string()),
    ];

    let root_entity = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(MenuUI)
        .id();

    let menu_layout_entity = templates::create_standard_menu_layout(
        &mut commands,
        &ui_assets,
        &menu_items,
        0,
        Some("Hub Services | Type code to join".to_string()),
        HubMenuMarker,
    );

    commands.entity(root_entity).add_child(menu_layout_entity);

    commands.entity(root_entity).with_children(|parent| {
        parent
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(50.0 * plt::UI_SCALE),
                    top: Val::Px(100.0 * plt::UI_SCALE),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    ..default()
                },
                HubCodeDisplay,
            ))
            .with_children(|node| {
                node.spawn((
                    Text::new("CODE: _____"),
                    TextFont {
                        font: ui_assets.font_kode_bold.clone(),
                        font_size: 48.0 * plt::FONT_SCALE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    HubCodeText,
                ));
                node.spawn((
                    Text::new("Type code using your keyboard"),
                    TextFont {
                        font: ui_assets.font_titillium_light.clone(),
                        font_size: 20.0 * plt::FONT_SCALE,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                ));
            });

        // Status label shown while pending / connecting.
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(50.0 * plt::UI_SCALE),
                top: Val::Px(260.0 * plt::UI_SCALE),
                ..default()
            },
            HubStatusLabel,
            Text::new(""),
            TextFont {
                font: ui_assets.font_kode_bold.clone(),
                font_size: 32.0 * plt::FONT_SCALE,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.85, 0.2)),
        ));
    });
}

pub fn hub_menu_event(
    mut click_events: MessageReader<MenuItemClicked>,
    mut escape_events: MessageReader<MenuEscapeEvent>,
    mut next_app_state: ResMut<NextState<UIContextState>>,
    menu_items: Query<(&HubMenuID, &MenuItemInteractive)>,
    hub_client: Res<HubClient>,
    mut hub_status: ResMut<HubStatus>,
    protocol_hash: Res<ClientProtocolHash>,
    runtime_installation_id: Option<Res<unprofile_core::profile::RuntimeInstallationId>>,
    room_code_input: Res<RoomCodeInput>,
) {
    if escape_events.read().next().is_some() {
        next_app_state.set(UIContextState::MainMenu);
        return;
    }

    for ev in click_events.read() {
        if ev.state != UIContextState::Hub {
            continue;
        }

        if hub_status.is_pending {
            continue;
        }

        if let Some((menu_id, _)) = menu_items
            .iter()
            .find(|(_, interactive)| interactive.identifier == ev.pos)
        {
            match menu_id {
                HubMenuID::CreateRoom => {
                    let player_uuid = runtime_installation_id
                        .as_ref()
                        .map(|x| x.0)
                        .unwrap_or_default();
                    let game_version = env!("CARGO_PKG_VERSION").to_string();
                    hub_client.create_room(player_uuid, game_version, protocol_hash.0.clone());
                    hub_status.is_pending = true;
                }
                HubMenuID::JoinRoom => {
                    let code = room_code_input.0.to_ascii_uppercase();
                    if code.len() == 5 {
                        let player_uuid = runtime_installation_id
                            .as_ref()
                            .map(|x| x.0)
                            .unwrap_or_default();
                        hub_client.join_room(code, player_uuid, protocol_hash.0.clone());
                        hub_status.is_pending = true;
                    }
                }
                HubMenuID::Back => {
                    next_app_state.set(UIContextState::MainMenu);
                }
            }
        }
    }
}

pub fn despawn_hub_ui(
    mut commands: Commands,
    query_ui: Query<Entity, With<MenuUI>>,
    query_cam: Query<Entity, With<MCamera>>,
) {
    for entity in &query_ui {
        commands.entity(entity).despawn();
    }
    for entity in &query_cam {
        commands.entity(entity).despawn();
    }
}

pub fn update_code_input(
    mut evr_char: MessageReader<KeyboardInput>,
    mut code_input: ResMut<RoomCodeInput>,
    mut q_text: Query<&mut Text, With<HubCodeText>>,
    hub_client: Res<HubClient>,
    mut hub_status: ResMut<HubStatus>,
    protocol_hash: Res<ClientProtocolHash>,
    runtime_installation_id: Option<Res<unprofile_core::profile::RuntimeInstallationId>>,
    time: Res<Time>,
    mut q_menu_root: Query<&mut MenuRoot>,
    mut last_len: Local<usize>,
    q_menu_items: Query<(&HubMenuID, &MenuItemInteractive)>,
) {
    for ev in evr_char.read() {
        if ev.state == bevy::input::ButtonState::Released {
            continue;
        }

        let key = &ev.key_code;

        if *key == KeyCode::Backspace {
            code_input.0.pop();
        } else if *key == KeyCode::Enter {
            let code = code_input.0.to_ascii_uppercase();
            if code.len() == 5 && !hub_status.is_pending {
                let player_uuid = runtime_installation_id
                    .as_ref()
                    .map(|x| x.0)
                    .unwrap_or_default();
                hub_client.join_room(code, player_uuid, protocol_hash.0.clone());
                hub_status.is_pending = true;
            }
        } else {
            let c = match key {
                KeyCode::KeyC => 'C',
                KeyCode::KeyD => 'D',
                KeyCode::KeyF => 'F',
                KeyCode::KeyG => 'G',
                KeyCode::KeyH => 'H',
                KeyCode::KeyJ => 'J',
                KeyCode::KeyK => 'K',
                KeyCode::KeyL => 'L',
                KeyCode::KeyM => 'M',
                KeyCode::KeyP => 'P',
                KeyCode::KeyR => 'R',
                KeyCode::KeyS => 'S',
                KeyCode::KeyT => 'T',
                KeyCode::KeyV => 'V',
                KeyCode::KeyW => 'W',
                KeyCode::KeyX => 'X',
                KeyCode::Digit2 => '2',
                KeyCode::Digit4 => '4',
                KeyCode::Digit7 => '7',
                KeyCode::Digit9 => '9',
                _ => '\0',
            };
            if c != '\0' && code_input.0.len() < 5 {
                code_input.0.push(c);
            }
        }
    }

    if code_input.0.len() == 5
        && *last_len < 5
        && let Some((_, item)) = q_menu_items
            .iter()
            .find(|(id, _)| **id == HubMenuID::JoinRoom)
        && let Ok(mut menu_root) = q_menu_root.single_mut()
    {
        menu_root.selected_item = item.identifier;
    }
    *last_len = code_input.0.len();

    let show_cursor = (time.elapsed_secs() * 2.0) as i32 % 2 == 0;

    for mut text in &mut q_text {
        let mut display = code_input.0.clone();
        if display.len() < 5 {
            let cursor = if show_cursor { '_' } else { ' ' };
            display.push(cursor);
            while display.len() < 5 {
                display.push('_');
            }
        }
        text.0 = format!("CODE: {}", display);
    }
}

/// Pick the best address from the hub-provided list: prefer IPv4 (no `[`)
/// for widest compatibility, fall back to the first IPv6 entry if no IPv4 is
/// available. Returns an empty string if `addrs` is empty (will surface as a
/// connection error downstream).
fn pick_best_addr(addrs: &[String]) -> String {
    addrs
        .iter()
        .find(|a| !a.starts_with('['))
        .or_else(|| addrs.first())
        .cloned()
        .unwrap_or_else(|| {
            warn!("Hub returned an empty address list; connection will fail");
            String::new()
        })
}

pub fn handle_hub_responses(
    mut hub_status: ResMut<HubStatus>,
    mut room_ident: ResMut<RoomIdentification>,
    mut hub_conn_events: MessageWriter<HubConnectionRequested>,
) {
    if let Some(resp) = hub_status.last_response.take() {
        match resp {
            HubResponse::RoomCreated(data) => {
                let addr = pick_best_addr(&data.addrs);
                info!("Hub: Room created: {} at {}", data.code, addr);
                room_ident.code = Some(data.code);
                room_ident.secret = Some(data.secret);
                hub_conn_events.write(HubConnectionRequested {
                    address: addr,
                    server_hostname: Some(data.server_hostname),
                    ticket: Some(data.ticket),
                });
                // Stay on the Hub screen; await_lobby_then_transition will move us to
                // Lobby once the server's LobbyInfo arrives via replication.
                hub_status.is_pending = false;
                hub_status.is_connecting = true;
            }
            HubResponse::RoomJoined(data) => {
                let addr = pick_best_addr(&data.addrs);
                info!("Hub: Room joined: {} at {}", data.code, addr);
                room_ident.code = Some(data.code);
                room_ident.secret = Some(data.secret);
                hub_conn_events.write(HubConnectionRequested {
                    address: addr,
                    server_hostname: Some(data.server_hostname),
                    ticket: Some(data.ticket),
                });
                // Same: stay on Hub, wait for LobbyInfo replication before going to Lobby.
                hub_status.is_pending = false;
                hub_status.is_connecting = true;
            }
            HubResponse::Error(e) => {
                error!("Hub error: {}", e);
                hub_status.is_connecting = false;
            }
            HubResponse::PingResult { .. } => {
                // Handled by update_hub_status; nothing to do in the Hub UI handler.
            }
        }
    }
}

/// While is_connecting, poll for a replicated LobbyInfo entity. Once one
/// arrives, start a short countdown before entering Lobby to let remaining
/// replication packets settle.
pub fn await_lobby_then_transition(
    mut hub_status: ResMut<HubStatus>,
    q_lobby: Query<(), With<LobbyInfo>>,
    mut next_app_state: ResMut<NextState<UIContextState>>,
    time: Res<Time>,
) {
    if !hub_status.is_connecting {
        return;
    }
    if let Some(ref mut remaining) = hub_status.lobby_ready_timer {
        *remaining -= time.delta_secs();
        if *remaining <= 0.0 {
            info!("Hub: lobby ready timer elapsed — transitioning to Lobby");
            hub_status.is_connecting = false;
            hub_status.lobby_ready_timer = None;
            next_app_state.set(UIContextState::Lobby);
        }
    } else if !q_lobby.is_empty() {
        info!("Hub: LobbyInfo received via replication — starting 1s countdown");
        hub_status.lobby_ready_timer = Some(1.0);
    }
}

/// Updates the Hub screen status label to give connecting feedback.
pub fn update_hub_status_label(
    hub_status: Res<HubStatus>,
    mut q_label: Query<(&mut Text, &mut TextColor), With<HubStatusLabel>>,
) {
    if !hub_status.is_changed() {
        return;
    }
    for (mut text, mut color) in &mut q_label {
        if let Some(error) = &hub_status.error_message {
            text.0 = format!("ERROR: {}", error);
            color.0 = Color::srgb(1.0, 0.5, 0.0); // Orange
        } else {
            color.0 = Color::srgb(1.0, 0.85, 0.2); // Default yellowish-orange
            text.0 = if hub_status.lobby_ready_timer.is_some() {
                "Ready! Entering lobby…".to_string()
            } else if hub_status.is_connecting {
                "CONNECTING… please wait".to_string()
            } else if hub_status.is_pending {
                "Contacting Hub…".to_string()
            } else {
                "".to_string()
            };
        }
    }
}
