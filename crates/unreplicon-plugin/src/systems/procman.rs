use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use std::io::{BufRead, Write};
use unhub_client::protocol::{DedicatedToProcMan, ProcManToDedicated};
use untypes_core::cli::{CliOptions, CliNetMode};

/// Bidirectional channel to the process manager over stdin/stdout.
///
/// Only present when `CliOptions::procman_channel == Some("stdin")`.
#[derive(Resource)]
pub(crate) struct ProcManChannel {
    /// Sender for outgoing messages to procman (player events, state sync).
    /// Used in Phase 2+ systems that report game state back to procman.
    #[allow(dead_code)]
    pub tx: Sender<DedicatedToProcMan>,
    pub rx: Receiver<ProcManToDedicated>,
}

/// Authentication state received from the process manager via `AssignRoom`.
///
/// Populated once procman assigns a room to this dedicated server.  Before
/// that point (when both fields are `None`) all incoming Renet connections are
/// rejected.
#[derive(Resource, Default)]
pub(crate) struct RoomAuth {
    /// The room code this server is currently hosting, or `None` if idle.
    pub room_code: Option<String>,
    /// HMAC-SHA256 key (hex-encoded 32 bytes) for validating JWT tickets.
    pub ticket_hmac_secret: Option<String>,
}

pub(super) fn app_setup(app: &mut App) {
    app.init_resource::<RoomAuth>();
    app.add_systems(Startup, setup_procman_system);
    app.add_systems(Update, update_procman_system);
}

fn setup_procman_system(mut commands: Commands, cli: Res<CliOptions>) {
    if cli.procman_channel.as_deref() != Some("stdin") {
        return;
    }

    let (tx_to_procman, rx_from_bevy) = crossbeam_channel::unbounded::<DedicatedToProcMan>();
    let (tx_to_bevy, rx_from_procman) = crossbeam_channel::unbounded::<ProcManToDedicated>();

    // Stdin reader thread — blocked on I/O, so it lives on its own OS thread.
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines().map_while(Result::ok) {
            if let Ok(msg) = serde_json::from_str::<ProcManToDedicated>(&line) {
                let _ = tx_to_bevy.send(msg);
            }
        }
    });

    // Stdout writer thread — receives messages from Bevy and forwards to stdout.
    std::thread::spawn(move || {
        let mut stdout = std::io::stdout();
        while let Ok(msg) = rx_from_bevy.recv() {
            if let Ok(line) = serde_json::to_string(&msg) {
                let _ = writeln!(stdout, "{}", line);
                let _ = stdout.flush();
            }
        }
    });

    // Send the initial Ready signal so procman knows the server is up.
    if let CliNetMode::PeerHost { port, .. } = cli.net_mode {
        let _ = tx_to_procman.send(DedicatedToProcMan::Ready { port });
    }

    commands.insert_resource(ProcManChannel {
        tx: tx_to_procman,
        rx: rx_from_procman,
    });
}

fn update_procman_system(
    procman: Option<Res<ProcManChannel>>,
    mut room_auth: ResMut<RoomAuth>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    let Some(procman) = procman.as_ref() else {
        return;
    };

    while let Ok(msg) = procman.rx.try_recv() {
        match msg {
            ProcManToDedicated::AssignRoom {
                room_code,
                secret: _,
                ticket_hmac_secret,
            } => {
                info!(
                    "ProcMan: Assigned room '{}'; ticket authentication active.",
                    room_code
                );
                room_auth.room_code = Some(room_code);
                room_auth.ticket_hmac_secret = Some(ticket_hmac_secret);
            }
            ProcManToDedicated::RenameRoom {
                new_code,
                new_secret: _,
            } => {
                info!("ProcMan: Room renamed to '{}'.", new_code);
                room_auth.room_code = Some(new_code);
                // Ticket secret stays the same — it is per-procman, not per-room.
            }
            ProcManToDedicated::WipeRoom { reason } => {
                info!("ProcMan: Room wiped: {}", reason);
                room_auth.room_code = None;
                room_auth.ticket_hmac_secret = None;
            }
            ProcManToDedicated::Shutdown { reason } => {
                info!("ProcMan: Shutdown requested: {}", reason);
                exit.write(bevy::app::AppExit::Success);
            }
        }
    }
}
