use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
use unhub_client::protocol::{RoomState, RoomSummary, RoomMetadata, ProcManMessage};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

use unhub_client::protocol::{ProcManToDedicated, DedicatedToProcMan};

pub struct ServerProcess {
    pub port: u16,
    pub room_code: Option<String>,
    pub secret: Option<String>,
    pub state: RoomState,
    pub player_count: u8,
    pub stdin_tx: tokio::sync::mpsc::UnboundedSender<ProcManToDedicated>,
}

pub struct ServerManager {
    pub config: crate::config::ProcManConfig,
    pub servers: Arc<Mutex<std::collections::HashMap<u16, ServerProcess>>>,
    pub hub_tx: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<ProcManMessage>>>>,
}

impl ServerManager {
    pub fn new(config: crate::config::ProcManConfig) -> Self {
        Self {
            config,
            servers: Arc::new(Mutex::new(std::collections::HashMap::new())),
            hub_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn maintain_pool(self: &Arc<Self>) -> anyhow::Result<()> {
        let mut servers = self.servers.lock().await;
        let idle_count = servers.values().filter(|s| s.room_code.is_none()).count();

        if idle_count < self.config.idle_pool_size {
            // Find next free port
            let mut port = self.config.port_range.0;
            while servers.contains_key(&port) {
                port += 1;
                if port > self.config.port_range.1 {
                    return Err(anyhow::anyhow!("No free ports in range"));
                }
            }

            info!("Spawning new idle server on port {}", port);
            let mut child = Command::new(&self.config.game_binary_path)
                .arg("--host")
                .arg(port.to_string())
                .arg("--procman-channel")
                .arg("stdin")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            let stdin = child.stdin.take().unwrap();
            let stdout = child.stdout.take().unwrap();
            let (stdin_tx, stdin_rx) = tokio::sync::mpsc::unbounded_channel();

            servers.insert(port, ServerProcess {
                port,
                room_code: None,
                secret: None,
                state: RoomState::Lobby,
                player_count: 0,
                stdin_tx,
            });

            let manager = self.clone();
            tokio::spawn(async move {
                if let Err(e) = manager.monitor_server(port, child, stdin, stdout, stdin_rx).await {
                    error!("Server on port {} error: {}", port, e);
                }
            });
        }
        Ok(())
    }

    async fn monitor_server(
        &self,
        port: u16,
        mut child: Child,
        mut stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
        mut stdin_rx: tokio::sync::mpsc::UnboundedReceiver<ProcManToDedicated>,
    ) -> anyhow::Result<()> {
        let mut reader = tokio::io::BufReader::new(stdout).lines();

        loop {
            tokio::select! {
                Some(msg) = stdin_rx.recv() => {
                    let line = serde_json::to_string(&msg)?;
                    stdin.write_all(format!("{}\n", line).as_bytes()).await?;
                    stdin.flush().await?;
                }
                result = child.wait() => {
                    let status = result?;
                    info!("Server on port {} exited with status {}", port, status);
                    let mut servers = self.servers.lock().await;
                    if let Some(s) = servers.remove(&port) {
                        if let Some(code) = s.room_code {
                            let tx_lock = self.hub_tx.lock().await;
                            if let Some(tx) = tx_lock.as_ref() {
                                let _ = tx.send(ProcManMessage::RoomClosed {
                                    room_code: code,
                                    port,
                                    reason: format!("Process exited with {}", status),
                                });
                            }
                        }
                    }
                    break;
                }
                line_res = reader.next_line() => {
                    match line_res {
                        Ok(Some(line)) => {
                            self.handle_server_output(port, &line).await?;
                        }
                        Ok(None) => break,
                        Err(e) => {
                            error!("Error reading server output on port {}: {}", port, e);
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_server_output(&self, port: u16, line: &str) -> anyhow::Result<()> {
        let msg: DedicatedToProcMan = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => return Ok(()),
        };

        match msg {
            DedicatedToProcMan::Ready { .. } => {
                info!("Server on port {} is ready", port);
            }
            DedicatedToProcMan::PlayerJoined { player_uuid } => {
                let mut servers = self.servers.lock().await;
                if let Some(s) = servers.get_mut(&port) {
                    s.player_count += 1;
                    if let Some(code) = &s.room_code {
                        let tx_lock = self.hub_tx.lock().await;
                        if let Some(tx) = tx_lock.as_ref() {
                            let _ = tx.send(ProcManMessage::PlayerJoined {
                                room_code: code.clone(),
                                player_uuid,
                                new_player_count: s.player_count,
                            });
                        }
                    }
                }
            }
            DedicatedToProcMan::PlayerLeft {
                player_uuid,
                remaining_count,
            } => {
                let mut servers = self.servers.lock().await;
                if let Some(s) = servers.get_mut(&port) {
                    s.player_count = remaining_count as u8;
                    if let Some(code) = &s.room_code {
                        let tx_lock = self.hub_tx.lock().await;
                        if let Some(tx) = tx_lock.as_ref() {
                            let _ = tx.send(ProcManMessage::PlayerLeft {
                                room_code: code.clone(),
                                player_uuid,
                                remaining_count: s.player_count,
                            });
                        }
                    }
                }
            }
            DedicatedToProcMan::StateChanged { state, metadata } => {
                let mut servers = self.servers.lock().await;
                if let Some(s) = servers.get_mut(&port) {
                    let old_state = s.state;
                    s.state = state;
                    if let Some(code) = &s.room_code {
                        let tx_lock = self.hub_tx.lock().await;
                        if let Some(tx) = tx_lock.as_ref() {
                            let _ = tx.send(ProcManMessage::RoomStateChanged {
                                room_code: code.clone(),
                                old_state,
                                new_state: state,
                                metadata,
                            });
                        }
                    }
                }
            }
            DedicatedToProcMan::RoomRenameRequest { reason } => {
                info!(
                    "Server on port {} requested room rename: {}",
                    port, reason
                );
            }
            DedicatedToProcMan::Exiting { reason } => {
                info!("Server on port {} is exiting: {}", port, reason);
            }
        }

        Ok(())
    }

    pub async fn assign_room(&self, room_code: String, secret: String, game_version: String) -> anyhow::Result<RoomSummary> {
        let mut servers = self.servers.lock().await;
        let server = servers.values_mut().find(|s| s.room_code.is_none()).ok_or_else(|| anyhow::anyhow!("No idle servers available"))?;

        server.room_code = Some(room_code.clone());
        server.secret = Some(secret.clone());

        let msg = ProcManToDedicated::AssignRoom {
            room_code: room_code.clone(),
            secret: secret.clone(),
        };
        server.stdin_tx.send(msg)?;

        Ok(RoomSummary {
            code: room_code,
            port: server.port,
            game_version,
            secret,
            state: RoomState::Lobby,
            player_count: 0,
            metadata: RoomMetadata { map: "".into(), difficulty: "".into() },
            server_id: self.config.installation_id,
        })
    }
}
