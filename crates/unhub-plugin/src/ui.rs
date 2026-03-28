use crate::hub_client::{HubClient, HubRequest, HubResponse, HubStatus};
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use uncommon_app_core::cli::{CliNetMode, CliOptions};
use uncommon_app_core::platform::plt;
use uncommon_app_core::states::AppState;
use unmenu_core::assets::MenuAssets;
use unmenu_core::components::{MCamera, MenuItemInteractive, MenuUI};
use unmenu_core::events::{MenuEscapeEvent, MenuItemClicked};
use unmenu_core::templates;
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
                ));
            });
    });
}

pub fn hub_menu_event(
    mut click_events: MessageReader<MenuItemClicked>,
    mut escape_events: MessageReader<MenuEscapeEvent>,
    mut next_app_state: ResMut<NextState<AppState>>,
    menu_items: Query<(&HubMenuID, &MenuItemInteractive)>,
    hub_client: Res<HubClient>,
    mut hub_status: ResMut<HubStatus>,
    runtime_installation_id: Option<Res<unprofile_core::profile::RuntimeInstallationId>>,
    room_code_input: Res<RoomCodeInput>,
) {
    if escape_events.read().next().is_some() {
        next_app_state.set(AppState::MainMenu);
        return;
    }

    for ev in click_events.read() {
        if ev.state != AppState::Hub {
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
                    let _ = hub_client.tx.send(HubRequest::CreateRoom {
                        player_uuid,
                        game_version,
                    });
                    hub_status.is_pending = true;
                }
                HubMenuID::JoinRoom => {
                    let code = room_code_input.0.to_ascii_uppercase();
                    if code.len() == 5 {
                        let player_uuid = runtime_installation_id
                            .as_ref()
                            .map(|x| x.0)
                            .unwrap_or_default();
                        let _ = hub_client
                            .tx
                            .send(HubRequest::JoinRoom { code, player_uuid });
                        hub_status.is_pending = true;
                    }
                }
                HubMenuID::Back => {
                    next_app_state.set(AppState::MainMenu);
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
    q_display: Query<&Children, With<HubCodeDisplay>>,
    mut q_text: Query<&mut Text>,
    hub_client: Res<HubClient>,
    mut hub_status: ResMut<HubStatus>,
    runtime_installation_id: Option<Res<unprofile_core::profile::RuntimeInstallationId>>,
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
                let _ = hub_client
                    .tx
                    .send(HubRequest::JoinRoom { code, player_uuid });
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

    for children in &q_display {
        for child in children.iter() {
            if let Ok(mut text) = q_text.get_mut(child.to_owned()) {
                let mut display = code_input.0.clone();
                while display.len() < 5 {
                    display.push('_');
                }
                text.0 = format!("CODE: {}", display);
            }
        }
    }
}

pub fn handle_hub_responses(
    mut hub_status: ResMut<HubStatus>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut cli: ResMut<CliOptions>,
    mut room_ident: ResMut<RoomIdentification>,
) {
    if let Some(resp) = hub_status.last_response.take() {
        match resp {
            HubResponse::RoomCreated(data) => {
                info!("Hub: Room created: {} at {}", data.code, data.addr);
                room_ident.code = Some(data.code);
                room_ident.secret = Some(data.secret);
                cli.net_mode = CliNetMode::Join {
                    address: data.addr.clone(),
                    ticket: Some(data.ticket.clone()),
                };
                next_app_state.set(AppState::Lobby);
            }
            HubResponse::RoomJoined(data) => {
                info!("Hub: Room joined: {} at {}", data.code, data.addr);
                room_ident.code = Some(data.code);
                room_ident.secret = Some(data.secret);
                cli.net_mode = CliNetMode::Join {
                    address: data.addr.clone(),
                    ticket: Some(data.ticket.clone()),
                };
                next_app_state.set(AppState::Lobby);
            }
            HubResponse::Error(e) => {
                error!("Hub error: {}", e);
            }
        }
    }
}
