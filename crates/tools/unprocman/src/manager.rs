use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use unhub_client::protocol::{
    DedicatedToProcMan, LibraryEntry, ProcManMessage, ProcManToDedicated, RoomMetadata, RoomState,
    RoomSummary,
};

#[derive(Clone)]
pub struct BinaryInfo {
    pub bundle_dir: PathBuf,
    pub binary_path: PathBuf,
    pub version: String,
    pub protocol_hash: String,
}

enum ServerControl {
    Kill { reason: String },
}

pub struct ServerProcess {
    pub port: u16,
    pub room_code: Option<String>,
    pub secret: Option<String>,
    pub state: RoomState,
    pub player_count: u8,
    pub assigned_at: Option<std::time::Instant>,
    pub spawned_at: std::time::Instant,
    pub has_been_joined: bool,
    pub ready: bool,
    pub stdin_tx: tokio::sync::mpsc::UnboundedSender<ProcManToDedicated>,
    control_tx: tokio::sync::mpsc::UnboundedSender<ServerControl>,
}

pub struct ServerManager {
    pub config: crate::config::ProcManConfig,
    pub servers: Arc<Mutex<HashMap<u16, ServerProcess>>>,
    pub library: Arc<Mutex<Vec<BinaryInfo>>>,
    pub hub_tx: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<ProcManMessage>>>>,
    pub spawn_in_progress: Arc<Mutex<bool>>,
}

impl ServerManager {
    pub fn new(config: crate::config::ProcManConfig) -> Self {
        Self {
            config,
            servers: Arc::new(Mutex::new(HashMap::new())),
            library: Arc::new(Mutex::new(Vec::new())),
            hub_tx: Arc::new(Mutex::new(None)),
            spawn_in_progress: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn scan_library(self: &Arc<Self>) -> anyhow::Result<()> {
        let root = PathBuf::from(&self.config.library_dir);
        if tokio::fs::metadata(&root).await.is_err() {
            warn!("Library directory does not exist: {}", root.display());
            let mut lib = self.library.lock().await;
            lib.clear();
            return Ok(());
        }

        let mut stack = vec![root.clone()];
        let mut found = Vec::new();

        while let Some(dir) = stack.pop() {
            let mut read = match tokio::fs::read_dir(&dir).await {
                Ok(r) => r,
                Err(e) => {
                    warn!("Failed to read library dir {}: {}", dir.display(), e);
                    continue;
                }
            };

            loop {
                let entry = match read.next_entry().await {
                    Ok(Some(e)) => e,
                    Ok(None) => break,
                    Err(e) => {
                        warn!("Failed to read library entry in {}: {}", dir.display(), e);
                        break;
                    }
                };
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                }
            }

            let version_file = dir.join("VERSION");
            let hash_file = dir.join("PROTOCOL_HASH");
            let has_version = tokio::fs::metadata(&version_file).await.is_ok();
            let has_hash = tokio::fs::metadata(&hash_file).await.is_ok();

            if has_version ^ has_hash {
                warn!(
                    "Skipping partial library bundle {} (VERSION/PROTOCOL_HASH mismatch)",
                    dir.display()
                );
                continue;
            }

            if !(has_version && has_hash) {
                continue;
            }

            let version = match tokio::fs::read_to_string(&version_file).await {
                Ok(v) => v.trim().to_string(),
                Err(e) => {
                    warn!("Failed reading VERSION in {}: {}", dir.display(), e);
                    continue;
                }
            };

            let protocol_hash = match tokio::fs::read_to_string(&hash_file)
                .await
                .ok()
                .map(|s| s.trim().to_string())
            {
                Some(h) => h,
                None => {
                    warn!("Invalid PROTOCOL_HASH in {}", dir.display());
                    continue;
                }
            };

            let binary_path = dir.join("unhaunter_dedicated");
            if tokio::fs::metadata(&binary_path).await.is_err() {
                warn!(
                    "Skipping bundle {} (missing binary {})",
                    dir.display(),
                    binary_path.display()
                );
                continue;
            }

            found.push(BinaryInfo {
                bundle_dir: dir.clone(),
                binary_path,
                version,
                protocol_hash,
            });
        }

        found.sort_by(|a, b| a.version.cmp(&b.version));
        info!("Library scan complete: {} valid bundles", found.len());

        let mut lib = self.library.lock().await;
        *lib = found;
        Ok(())
    }

    pub async fn get_library_entries(&self) -> Vec<LibraryEntry> {
        let lib = self.library.lock().await;
        lib.iter()
            .map(|b| LibraryEntry {
                version: b.version.clone(),
                protocol_hash: b.protocol_hash.clone(),
            })
            .collect()
    }

    pub async fn maintain_pool(self: &Arc<Self>) -> anyhow::Result<()> {
        let mut servers = self.servers.lock().await;

        // Reclaim unused assigned rooms (state cleanup, not idle pooling)
        let mut to_wipe = Vec::new();
        let mut to_kill = Vec::new();
        for s in servers.values() {
            if let (Some(code), 0, Some(assigned_at)) =
                (&s.room_code, s.player_count, s.assigned_at)
            {
                let timeout_secs = if s.has_been_joined { 300 } else { 5 };
                if assigned_at.elapsed().as_secs() > timeout_secs {
                    to_wipe.push((s.port, code.clone()));
                }
            }

            if s.spawned_at.elapsed().as_secs() > 2 * 60 * 60 {
                to_kill.push(s.port);
            }
        }

        for (port, code) in to_wipe {
            warn!("Reclaiming unused room {} on port {}", code, port);
            if let Some(s) = servers.get_mut(&port) {
                s.room_code = None;
                s.secret = None;
                s.assigned_at = None;
                let _ = s.stdin_tx.send(ProcManToDedicated::WipeRoom {
                    reason: "Unused room reclaimed".to_string(),
                });
                if let Some(tx) = self.hub_tx.lock().await.as_ref() {
                    let _ = tx.send(ProcManMessage::RoomClosed {
                        room_code: code,
                        port,
                        reason: "Reclaimed (unused)".to_string(),
                    });
                }
            } else {
                warn!(
                    "Expected server on port {} during reclaim, but none found",
                    port
                );
            }
        }

        for port in to_kill {
            if let Some(s) = servers.get(&port) {
                warn!("Killing server on port {} (TTL exceeded 2h)", port);
                let _ = s.control_tx.send(ServerControl::Kill {
                    reason: "TTL exceeded 2h".to_string(),
                });
            } else {
                warn!("TTL kill requested for missing server on port {}", port);
            }
        }

        Ok(())
    }

    fn find_free_port_locked(&self, servers: &HashMap<u16, ServerProcess>) -> anyhow::Result<u16> {
        let mut port = self.config.port_range.0;
        while servers.contains_key(&port) {
            port += 1;
            if port > self.config.port_range.1 {
                return Err(anyhow::anyhow!("No free ports in configured range"));
            }
        }
        Ok(port)
    }

    fn find_binary_for_version(&self, version: &str, library: &[BinaryInfo]) -> Option<BinaryInfo> {
        library.iter().find(|b| b.version == version).cloned()
    }

    /// Wrapper that ensures cleanup always runs on all exit paths.
    async fn monitor_server(
        &self,
        port: u16,
        child: Child,
        stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
        stderr: tokio::process::ChildStderr,
        stdin_rx: tokio::sync::mpsc::UnboundedReceiver<ProcManToDedicated>,
        control_rx: tokio::sync::mpsc::UnboundedReceiver<ServerControl>,
    ) -> anyhow::Result<()> {
        let result = self
            .monitor_server_loop(port, child, stdin, stdout, stderr, stdin_rx, control_rx)
            .await;

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
        mut control_rx: tokio::sync::mpsc::UnboundedReceiver<ServerControl>,
    ) -> anyhow::Result<()> {
        let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();
        let mut stderr_reader = tokio::io::BufReader::new(stderr).lines();

        loop {
            tokio::select! {
                Some(ctrl) = control_rx.recv() => {
                    match ctrl {
                        ServerControl::Kill { reason } => {
                            warn!("Killing server on port {}: {}", port, reason);
                            if let Err(e) = child.kill().await {
                                error!("Failed to kill server on port {}: {}", port, e);
                            }
                        }
                    }
                }
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
                        Ok(Some(line)) => self.log_child_line(port, line.trim(), true),
                        Ok(None) => {},
                        Err(e) => error!("Error reading server stderr on port {}: {}", port, e),
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
        if line.contains(" ERROR ") {
            error!("{} {}", prefix, line);
        } else if line.contains(" WARN ") {
            warn!("{} {}", prefix, line);
        } else if line.contains(" DEBUG ") || line.contains(" TRACE ") {
            debug!("{} {}", prefix, line);
        } else if is_stderr {
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
                let mut servers = self.servers.lock().await;
                if let Some(s) = servers.get_mut(&port) {
                    s.ready = true;
                } else {
                    warn!("Received Ready for missing server on port {}", port);
                }
            }
            DedicatedToProcMan::PlayerJoined { player_uuid } => {
                let mut servers = self.servers.lock().await;
                if let Some(s) = servers.get_mut(&port) {
                    s.player_count = s.player_count.saturating_add(1);
                    s.has_been_joined = true;
                    if let Some(code) = &s.room_code
                        && let Some(tx) = self.hub_tx.lock().await.as_ref()
                    {
                        let _ = tx.send(ProcManMessage::PlayerJoined {
                            room_code: code.clone(),
                            player_uuid,
                            new_player_count: s.player_count,
                        });
                    }
                } else {
                    warn!("PlayerJoined for missing server on port {}", port);
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
                    if let Some(code) = &s.room_code
                        && let Some(tx) = self.hub_tx.lock().await.as_ref()
                    {
                        let _ = tx.send(ProcManMessage::PlayerLeft {
                            room_code: code.clone(),
                            player_uuid,
                            remaining_count: s.player_count,
                        });
                    }
                } else {
                    warn!("PlayerLeft for missing server on port {}", port);
                }
            }
            DedicatedToProcMan::StateChanged { state, metadata } => {
                let mut servers = self.servers.lock().await;
                if let Some(s) = servers.get_mut(&port) {
                    let old_state = s.state;
                    s.state = state;
                    if let Some(code) = &s.room_code
                        && let Some(tx) = self.hub_tx.lock().await.as_ref()
                    {
                        let _ = tx.send(ProcManMessage::RoomStateChanged {
                            room_code: code.clone(),
                            old_state,
                            new_state: state,
                            metadata,
                        });
                    }
                } else {
                    warn!("StateChanged for missing server on port {}", port);
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
        self: &Arc<Self>,
        room_code: String,
        secret: String,
        target_version: String,
    ) -> anyhow::Result<RoomSummary> {
        {
            let mut in_progress = self.spawn_in_progress.lock().await;
            if *in_progress {
                warn!("Rejecting CreateRoom while spawn is in progress");
                return Err(anyhow::anyhow!("Spawn already in progress"));
            }
            *in_progress = true;
        }

        let result = async {
            let library = self.library.lock().await;
            let binary = self
                .find_binary_for_version(&target_version, &library)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "No library bundle found for target version {}",
                        target_version
                    )
                })?;

            let mut servers = self.servers.lock().await;
            if servers.len() >= self.config.max_total_instances {
                warn!(
                    "Rejecting CreateRoom: max_total_instances reached ({})",
                    self.config.max_total_instances
                );
                return Err(anyhow::anyhow!("No capacity available"));
            }

            let port = self.find_free_port_locked(&servers)?;
            info!(
                "Spawning on-demand server for target version {} on port {} using {}",
                target_version,
                port,
                binary.binary_path.display()
            );

            let mut cmd = Command::new(&binary.binary_path);
            cmd.current_dir(&binary.bundle_dir)
                .arg("--host")
                .arg(port.to_string())
                .arg("--procman-channel")
                .arg("stdin");

            if let Some(cert) = &self.config.cert_file {
                cmd.arg("--cert").arg(cert);
            }
            if let Some(key) = &self.config.key_file {
                cmd.arg("--key").arg(key);
            }
            if self.config.skip_ssl_verification {
                cmd.arg("--skip-ssl-verification");
            }

            cmd.arg("-vv")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .env("NO_COLOR", "1")
                .env("TERM", "dumb");

            if self.config.debug_children {
                cmd.env("RUST_LOG", "debug");
            } else {
                cmd.env_remove("RUST_LOG");
            }

            let mut child = cmd.spawn()?;
            let stdin = child
                .stdin
                .take()
                .ok_or_else(|| anyhow::anyhow!("Spawned child missing stdin"))?;
            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| anyhow::anyhow!("Spawned child missing stdout"))?;
            let stderr = child
                .stderr
                .take()
                .ok_or_else(|| anyhow::anyhow!("Spawned child missing stderr"))?;

            let (stdin_tx, stdin_rx) = tokio::sync::mpsc::unbounded_channel();
            let (control_tx, control_rx) = tokio::sync::mpsc::unbounded_channel();
            let ticket_hmac_secret = self.config.ticket_hmac_secret.clone();

            servers.insert(
                port,
                ServerProcess {
                    port,
                    room_code: Some(room_code.clone()),
                    secret: Some(secret.clone()),
                    state: RoomState::Lobby,
                    player_count: 0,
                    assigned_at: Some(std::time::Instant::now()),
                    spawned_at: std::time::Instant::now(),
                    has_been_joined: false,
                    ready: false,
                    stdin_tx: stdin_tx.clone(),
                    control_tx: control_tx.clone(),
                },
            );
            drop(servers);

            let manager = self.clone();
            tokio::spawn(async move {
                if let Err(e) = manager
                    .monitor_server(port, child, stdin, stdout, stderr, stdin_rx, control_rx)
                    .await
                {
                    error!("Server on port {} error: {}", port, e);
                }
            });

            stdin_tx.send(ProcManToDedicated::AssignRoom {
                room_code: room_code.clone(),
                secret: secret.clone(),
                ticket_hmac_secret,
            })?;

            let wait_ready = async {
                loop {
                    {
                        let servers = self.servers.lock().await;
                        let Some(server) = servers.get(&port) else {
                            return Err(anyhow::anyhow!(
                                "Spawned server disappeared before Ready on port {}",
                                port
                            ));
                        };
                        if server.ready {
                            break;
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                Ok::<(), anyhow::Error>(())
            };

            match tokio::time::timeout(std::time::Duration::from_secs(10), wait_ready).await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    let _ = control_tx.send(ServerControl::Kill {
                        reason: "Spawn failed before ready".to_string(),
                    });
                    return Err(e);
                }
                Err(_) => {
                    let _ = control_tx.send(ServerControl::Kill {
                        reason: "Ready timeout (10s)".to_string(),
                    });
                    return Err(anyhow::anyhow!("Spawn timeout waiting for Ready"));
                }
            }

            Ok(RoomSummary {
                code: room_code,
                port,
                game_version: target_version,
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
        .await;

        let mut in_progress = self.spawn_in_progress.lock().await;
        *in_progress = false;

        result
    }
}
