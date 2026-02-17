use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use std::io::{BufRead, Write};
use unhub_client::protocol::{DedicatedToProcMan, ProcManToDedicated, RoomMetadata, RoomState};
use unnet_core::resources::{LobbyData, RoomIdentification};
use untypes_core::cli::{CliOptions, NetMode};
use untypes_core::states::AppState;

#[derive(Resource)]
pub(crate) struct ProcManChannel {
    pub tx: Sender<DedicatedToProcMan>,
    pub rx: Receiver<ProcManToDedicated>,
}

pub(crate) fn setup_procman_system(mut commands: Commands, cli: Res<CliOptions>) {
    if cli.procman_channel.as_deref() != Some("stdin") {
        return;
    }

    let (tx_to_procman, rx_from_bevy) = crossbeam_channel::unbounded::<DedicatedToProcMan>();
    let (tx_to_bevy, rx_from_procman) = crossbeam_channel::unbounded::<ProcManToDedicated>();

    // Thread for reading from stdin
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines().map_while(Result::ok) {
            if let Ok(msg) = serde_json::from_str::<ProcManToDedicated>(&line) {
                let _ = tx_to_bevy.send(msg);
            }
        }
    });

    // Thread for writing to stdout
    std::thread::spawn(move || {
        let mut stdout = std::io::stdout();
        while let Ok(msg) = rx_from_bevy.recv() {
            if let Ok(line) = serde_json::to_string(&msg) {
                let _ = writeln!(stdout, "{}", line);
                let _ = stdout.flush();
            }
        }
    });

    commands.insert_resource(ProcManChannel {
        tx: tx_to_procman.clone(),
        rx: rx_from_procman,
    });

    // Send initial Ready message
    if let NetMode::Host { port, .. } = cli.net_mode {
        let _ = tx_to_procman.send(DedicatedToProcMan::Ready { port });
    }
}

pub(crate) fn update_procman_system(
    mut procman: Option<ResMut<ProcManChannel>>,
    mut room_ident: ResMut<RoomIdentification>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut conn: ResMut<crate::resources::NetworkConn>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    let Some(procman) = procman.as_mut() else {
        return;
    };

    while let Ok(msg) = procman.rx.try_recv() {
        match msg {
            ProcManToDedicated::AssignRoom { room_code, secret } => {
                info!("ProcMan: Assigned room code {} with secret", room_code);
                room_ident.code = Some(room_code);
                room_ident.secret = Some(secret);
                next_app_state.set(AppState::Lobby);
            }
            ProcManToDedicated::RenameRoom {
                new_code,
                new_secret,
            } => {
                info!("ProcMan: Renamed room to {} with new secret", new_code);
                room_ident.code = Some(new_code);
                room_ident.secret = Some(new_secret);
            }
            ProcManToDedicated::WipeRoom { reason } => {
                info!("ProcMan: Wiping room: {}", reason);
                room_ident.code = None;
                room_ident.secret = None;
                *conn = crate::resources::NetworkConn::Disconnected;
                next_app_state.set(AppState::Hub);
            }
            ProcManToDedicated::Shutdown { reason } => {
                info!("ProcMan: Shutdown requested: {}", reason);
                exit.write(bevy::app::AppExit::Success);
            }
        }
    }
}

pub(crate) fn procman_state_sync_system(
    procman: Option<Res<ProcManChannel>>,
    app_state: Res<State<AppState>>,
    lobby_data: Res<LobbyData>,
    mut last_state: Local<Option<(AppState, Option<String>, String)>>,
) {
    let Some(procman) = procman.as_ref() else {
        return;
    };

    let current_map = lobby_data.selected_map.clone().unwrap_or_default();
    let current_difficulty = lobby_data.selected_difficulty.clone();
    let current_state = *app_state.get();

    let state_changed = last_state.as_ref().is_none_or(|(s, m, d)| {
        *s != current_state || m.as_ref() != Some(&current_map) || d != &current_difficulty
    });

    if state_changed {
        let room_state = match current_state {
            AppState::InGame => RoomState::InGame,
            _ => RoomState::Lobby,
        };

        let metadata = RoomMetadata {
            map: current_map.clone(),
            difficulty: current_difficulty.clone(),
        };

        let _ = procman.tx.send(DedicatedToProcMan::StateChanged {
            state: room_state,
            metadata,
        });

        *last_state = Some((current_state, Some(current_map), current_difficulty));
    }
}

pub(crate) fn procman_player_events_system(
    procman: Option<Res<ProcManChannel>>,
    mut ev_joined: MessageReader<unnet_core::messages::PlayerJoinedEvent>,
    mut ev_left: MessageReader<unnet_core::messages::NetworkDisconnectEvent>,
    lobby_data: Res<LobbyData>,
) {
    let Some(procman) = procman.as_ref() else {
        return;
    };

    for ev in ev_joined.read() {
        let _ = procman.tx.send(DedicatedToProcMan::PlayerJoined {
            player_uuid: ev.uuid,
        });
    }

    for ev in ev_left.read() {
        // Note: the client might already be removed from the `clients` list by the time we get here
        // depending on system ordering. network_io_system removes them.
        // However, DedicatedToProcMan::PlayerLeft needs the remaining_count.
        let remaining_count = lobby_data.players.iter().filter(|p| p.connected).count();

        let _ = procman.tx.send(DedicatedToProcMan::PlayerLeft {
            player_uuid: ev.uuid.unwrap_or_default(),
            remaining_count,
        });
    }
}

pub(crate) fn dynamic_tick_rate_system(
    app_state: Res<State<AppState>>,
    lobby_data: Res<LobbyData>,
    cli: Res<CliOptions>,
    mut last_tick: Local<Option<std::time::Instant>>,
    mut fixed_time: ResMut<Time<Fixed>>,
) {
    if !cli.dedicated {
        return;
    }

    let current_state = *app_state.get();
    let player_count = lobby_data.players.iter().filter(|p| p.connected).count();

    let (target_hz, fixed_hz) = match current_state {
        AppState::Hub | AppState::Loading | AppState::MainMenu => (1.0, 1.0),
        AppState::Lobby => {
            if player_count <= 1 {
                (1.0, 1.0)
            } else {
                (5.0, 5.0)
            }
        }
        AppState::InGame => {
            if player_count <= 1 {
                (15.0, 15.0)
            } else {
                (60.0, 15.0) // Keep physics at 15Hz even if display/network is 60Hz
            }
        }
        _ => (1.0, 1.0),
    };

    fixed_time.set_timestep(std::time::Duration::from_secs_f32(1.0 / fixed_hz));

    let now = std::time::Instant::now();
    if let Some(last) = *last_tick {
        let elapsed = now - last;
        let target_period = std::time::Duration::from_secs_f32(1.0 / target_hz);
        if elapsed < target_period {
            std::thread::sleep(target_period - elapsed);
        }
    }
    *last_tick = Some(std::time::Instant::now());
}
