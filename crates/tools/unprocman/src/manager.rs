use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use unhub_client::protocol::{ProcManMessage, RoomMetadata, RoomState, RoomSummary};

use unhub_client::protocol::{DedicatedToProcMan, ProcManToDedicated};

pub struct ServerProcess {
    pub port: u16,
    pub room_code: Option<String>,
    pub secret: Option<String>,
    pub state: RoomState,
    pub player_count: u8,
    pub assigned_at: Option<std::time::Instant>,
    pub has_been_joined: bool,
    pub stdin_tx: tokio::sync::mpsc::UnboundedSender<ProcManToDedicated>,
    /// The HMAC secret passed to the game server so it can validate JWT tickets.
    pub ticket_hmac_secret: Option<String>,
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

        // Reclaim unused rooms
        let mut to_wipe = Vec::new();
        for s in servers.values() {
            if let (Some(code), 0, Some(assigned_at)) =
                (&s.room_code, s.player_count, s.assigned_at)
            {
                let timeout_secs = if s.has_been_joined {
                    300 // 5 minutes if it was once joined
                } else {
                    5 // Fast Expiry: 5 seconds if never joined (Phase 1.4)
                };

                if assigned_at.elapsed().as_secs() > timeout_secs {
                    to_wipe.push((s.port, code.clone()));
                }
            }
        }

        for (port, code) in to_wipe {
            warn!(
                "Reclaiming unused room {} on port {} (no players for 30s)",
                code, port
            );
            if let Some(s) = servers.get_mut(&port) {
                s.room_code = None;
                s.secret = None;
                s.assigned_at = None;
                let _ = s.stdin_tx.send(ProcManToDedicated::WipeRoom {
                    reason: "Unused for >30s".to_string(),
                });

                // Notify Hub
                if let Some(tx) = self.hub_tx.lock().await.as_ref() {
                    let _ = tx.send(ProcManMessage::RoomClosed {
                        room_code: code,
                        port,
                        reason: "Reclaimed (unused)".to_string(),
                    });
                }
            }
        }

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
            let mut cmd = Command::new(&self.config.game_binary_path);
            cmd.arg("--host")
                .arg(port.to_string())
                .arg("--procman-channel")
                .arg("stdin")
                .arg("-vv")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .env("NO_COLOR", "1")
                .env("TERM", "dumb")
                // Clear any inherited RUST_LOG so the game binary uses its own
                // built-in log filter (build_log_filter), which correctly caps
                // high-volume third-party crates like bevy_replicon.
                .env_remove("RUST_LOG");

            if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
                let path = std::path::Path::new(&manifest_dir);
                if let Some(root) = path
                    .parent()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.parent())
                {
                    cmd.env("CARGO_MANIFEST_DIR", root);
                }
            }

            let mut child = cmd.spawn()?;

            let stdin = child.stdin.take().unwrap();
            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();
            let (stdin_tx, stdin_rx) = tokio::sync::mpsc::unbounded_channel();

            servers.insert(
                port,
                ServerProcess {
                    port,
                    room_code: None,
                    secret: None,
                    state: RoomState::Lobby,
                    player_count: 0,
                    assigned_at: None,
                    has_been_joined: false,
                    stdin_tx,
                    ticket_hmac_secret: None,
                },
            );

            let manager = self.clone();
            tokio::spawn(async move {
                if let Err(e) = manager
                    .monitor_server(port, child, stdin, stdout, stderr, stdin_rx)
                    .await
                {
                    error!("Server on port {} error: {}", port, e);
                }
            });
        }
        Ok(())
    }

    /// Wrapper that ensures cleanup always runs on all exit paths (normal exit,
    /// error, `?` propagation, stdout EOF, etc). The actual monitoring loop is
    /// in `monitor_server_loop`.
    async fn monitor_server(
        &self,
        port: u16,
        child: Child,
        stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
        stderr: tokio::process::ChildStderr,
        stdin_rx: tokio::sync::mpsc::UnboundedReceiver<ProcManToDedicated>,
    ) -> anyhow::Result<()> {
        let result = self
            .monitor_server_loop(port, child, stdin, stdout, stderr, stdin_rx)
            .await;

        // Cleanup on ALL exit paths — wrapper guarantees this runs
        let mut servers = self.servers.lock().await;
        if let Some(s) = servers.remove(&port)
            && let Some(code) = s.room_code
            && let Some(tx) = self.hub_tx.lock().await.as_ref()
        {
            let reason = match &result {
                Ok(()) => "Server exited normally".to_string(),
                Err(e) => format!("Server error: {}", e),
            };
            let _ = tx.send(ProcManMessage::RoomClosed {
                room_code: code,
                port,
                reason,
            });
        }

        result
    }

    async fn monitor_server_loop(
        &self,
        port: u16,
        mut child: Child,
        mut stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
        stderr: tokio::process::ChildStderr,
        mut stdin_rx: tokio::sync::mpsc::UnboundedReceiver<ProcManToDedicated>,
    ) -> anyhow::Result<()> {
        let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();
        let mut stderr_reader = tokio::io::BufReader::new(stderr).lines();

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
                    break;
                }
                line_res = stdout_reader.next_line() => {
                    match line_res {
                        Ok(Some(line)) => {
                            let trimmed = line.trim();
                            if trimmed.starts_with('{') && trimmed.ends_with('}') {
                                if let Err(e) = self.handle_server_output(port, trimmed).await {
                                    error!("Protocol error on port {}: {} (line: {})", port, e, trimmed);
                                }
                            } else {
                                self.log_child_line(port, trimmed, false);
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            error!("Error reading server stdout on port {}: {}", port, e);
                            break;
                        }
                    }
                }
                line_res = stderr_reader.next_line() => {
                    match line_res {
                        Ok(Some(line)) => {
                            self.log_child_line(port, line.trim(), true);
                        }
                        Ok(None) => {}, // stderr closed, but stdout might still be alive
                        Err(e) => {
                            error!("Error reading server stderr on port {}: {}", port, e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn log_child_line(&self, port: u16, line: &str, is_stderr: bool) {
        if line.is_empty() {
            return;
        }

        let prefix = format!("[Port {}]", port);

        // Detect log level from Bevy's format (e.g., "... ERROR ...")
        if line.contains(" ERROR ") {
            error!("{} {}", prefix, line);
        } else if line.contains(" WARN ") {
            warn!("{} {}", prefix, line);
        } else if line.contains(" DEBUG ") {
            debug!("{} {}", prefix, line);
        } else if line.contains(" TRACE ") {
            debug!("{} {}", prefix, line); // Map trace to debug for unprocman
        } else if is_stderr {
            // Stderr lines without explicit level are likely warnings/errors
            warn!("{} {}", prefix, line);
        } else {
            info!("{} {}", prefix, line);
        }
    }

    async fn handle_server_output(&self, port: u16, line: &str) -> anyhow::Result<()> {
        let msg: DedicatedToProcMan = serde_json::from_str(line)?;

        match msg {
            DedicatedToProcMan::Ready { .. } => {
                info!("Server on port {} is ready", port);
            }
            DedicatedToProcMan::PlayerJoined { player_uuid } => {
                let mut servers = self.servers.lock().await;
                if let Some(s) = servers.get_mut(&port) {
                    s.player_count += 1;
                    s.has_been_joined = true;
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
                    let old_count = s.player_count;
                    s.player_count = remaining_count as u8;
                    if old_count > 0 && s.player_count == 0 {
                        s.assigned_at = Some(std::time::Instant::now());
                    }
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
                info!("Server on port {} requested room rename: {}", port, reason);
            }
            DedicatedToProcMan::Exiting { reason } => {
                info!("Server on port {} is exiting: {}", port, reason);
            }
        }

        Ok(())
    }

    pub async fn assign_room(
        &self,
        room_code: String,
        secret: String,
        game_version: String,
    ) -> anyhow::Result<RoomSummary> {
        let mut servers = self.servers.lock().await;
        let server = servers
            .values_mut()
            .find(|s| s.room_code.is_none())
            .ok_or_else(|| anyhow::anyhow!("No idle servers available"))?;

        server.room_code = Some(room_code.clone());
        server.secret = Some(secret.clone());
        server.assigned_at = Some(std::time::Instant::now());
        server.player_count = 0;
        server.has_been_joined = false;

        let ticket_hmac_secret = self.config.ticket_hmac_secret.clone();
        server.ticket_hmac_secret = Some(ticket_hmac_secret.clone());

        let msg = ProcManToDedicated::AssignRoom {
            room_code: room_code.clone(),
            secret: secret.clone(),
            ticket_hmac_secret,
        };
        server.stdin_tx.send(msg)?;

        Ok(RoomSummary {
            code: room_code,
            port: server.port,
            game_version,
            secret,
            state: RoomState::Lobby,
            player_count: 0,
            metadata: RoomMetadata {
                map: "".into(),
                difficulty: "".into(),
            },
            server_id: self.config.installation_id,
        })
    }
}
