use bevy::prelude::*;
use std::io::{BufRead, Write};
use unhub_client::protocol::{DedicatedToProcMan, ProcManToDedicated};

use crate::resources::{ProcManChannel, ProcManConfig, RoomAuth};

pub(super) fn app_setup(app: &mut App) {
    app.init_resource::<RoomAuth>();
    app.add_systems(Startup, setup_procman_system);
    app.add_systems(Update, (update_procman_system, idle_timeout_system));
}

fn setup_procman_system(mut commands: Commands, procman_config: Res<ProcManConfig>) {
    if procman_config.procman_channel.as_deref() != Some("stdin") {
        return;
    }

    let (tx_to_procman, rx_from_bevy) = crossbeam_channel::unbounded::<DedicatedToProcMan>();
    let (tx_to_bevy, rx_from_procman) = crossbeam_channel::unbounded::<ProcManToDedicated>();

    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines().map_while(Result::ok) {
            if let Ok(msg) = serde_json::from_str::<ProcManToDedicated>(&line) {
                let _ = tx_to_bevy.send(msg);
            }
        }
    });

    std::thread::spawn(move || {
        let mut stdout = std::io::stdout();
        while let Ok(msg) = rx_from_bevy.recv() {
            if let Ok(line) = serde_json::to_string(&msg) {
                let _ = writeln!(stdout, "{}", line);
                let _ = stdout.flush();
            }
        }
    });

    let _ = tx_to_procman.send(DedicatedToProcMan::Ready {
        port: procman_config.port,
    });

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

/// Automatically shut down the dedicated server if it's empty and was created via unhub.
fn idle_timeout_system(
    time: Res<Time>,
    procman: Option<Res<ProcManChannel>>,
    room_auth: Res<RoomAuth>,
    server: Option<Res<bevy_renet2::prelude::RenetServer>>,
    mut idle_timer: Local<f32>,
    mut exit_sent: Local<bool>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    // Only apply to unhub-managed servers (those with a procman channel and an assigned room).
    if procman.is_none() || room_auth.room_code.is_none() {
        *idle_timer = 0.0;
        return;
    }

    let Some(server) = server else {
        *idle_timer = 0.0;
        return;
    };

    let client_count = server.clients_id().len();

    if client_count == 0 {
        *idle_timer += time.delta_secs();
        if *idle_timer >= 5.0 && !*exit_sent {
            info!("Dedicated server is empty and idle for 5s; shutting down.");
            exit.write(bevy::app::AppExit::Success);
            *exit_sent = true;
        }
    } else {
        *idle_timer = 0.0;
    }
}
