# Hosting an Unhaunter Multiplayer Hub

This guide is for community members who want to run their own Unhaunter multiplayer infrastructure — a Hub, Process
Manager, and Dedicated Game Servers — so that players can create and join rooms using short codes instead of sharing IP
addresses.

**Status: Early / Unstable.** This infrastructure was implemented recently and has not been battle-tested in production.
Expect rough edges. If you run into problems, please report them on [GitHub](https://github.com/deavid/unhaunter/issues)
or [Discord](https://discord.gg/Ux7CGfvVtV).

---

## What You're Deploying

Three separate programs work together:

```
Players ──HTTPS/HTTP──► Hub (unhub)
                         │
                         │  persistent TCP (ProcMan connects OUT to Hub)
                         │
                    ProcMan (unprocman)
                         │
                         │  stdin/stdout (ProcMan spawns and manages these)
                         │
               Dedicated Servers (unhaunter_dedicated)
                         ▲
                         │
                    Players connect directly via TCP
                    (after Hub gives them the address)
```

| Program                   | What it does                                                                                                                                                                                                                                                              |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`unhub`**               | The Hub. A lightweight HTTP API server. Players talk to it to create/join rooms. It never runs game code. It routes players to dedicated servers via room codes.                                                                                                          |
| **`unprocman`**           | The Process Manager. Runs on the same machine as the game servers. Connects **outbound** to the Hub (no inbound ports needed for ProcMan itself). Spawns, monitors, and recycles `unhaunter_dedicated` processes. Maintains a pool of idle servers ready to accept rooms. |
| **`unhaunter_dedicated`** | The actual game server. Headless Bevy — no GPU, no window, no audio. One process = one room = one TCP port. Players connect to it directly after the Hub tells them where it is.                                                                                          |

**Key architectural property:** ProcMan connects _outbound_ to the Hub. The Hub never initiates connections to ProcMan.
This means ProcMan machines don't need special firewall rules for the Hub link — only the game port range needs to be
open for player traffic.

---

## Prerequisites

- A Linux machine (or VPS). The Hub is very lightweight (~10MB RAM for the Hub process itself). The dedicated servers
  are heavier (Bevy's scheduler uses ~10% CPU per idle instance on a 2-vCore VPS — this is a known issue).
- Rust toolchain. See [INSTALLING_DEPS.md](INSTALLING_DEPS.md) for Bevy's system dependencies.

> **TODO:** There is currently no release process for the Hub/ProcMan/Dedicated binaries. The [Justfile](Justfile) and
> [RELEASING.md](RELEASING.md) only cover the game client (`unhaunter_game`). You must build from source. This needs to
> be fixed — prebuilt binaries and/or container images should be provided. See also: the `assets/` directory must be
> available to `unhaunter_dedicated` at runtime (it loads map data), but there is no packaging step that bundles assets
> with the server binary.

---

## Step 1: Build Everything

From the repository root:

```bash
cargo build --release --bin unhub --bin unprocman --bin unhaunter_dedicated
```

This produces three binaries:

- `target/release/unhub`
- `target/release/unprocman`
- `target/release/unhaunter_dedicated`

> **TODO:** `--release` builds are slow (full Bevy compilation). There are no feature flags to produce a lighter
> dedicated server build. The `unhaunter_dedicated` binary links against Bevy, which pulls in significant dependencies
> even in headless mode.

---

## Step 2: Start the Hub (`unhub`)

The Hub must be running first. Everything else connects to it.

```bash
cd /path/to/your/deployment/hub
/path/to/unhub
```

On first run, it generates a default `hub_config.ron` in the current working directory.

**Expected output on success:**

```
INFO unhub: Hub API listening on 0.0.0.0:3000
INFO unhub::procman: ProcMan listener running on 0.0.0.0:11000
```

**If you see nothing or an error:** Check that ports 3000 and 11000 are not already in use. The Hub binds to
`0.0.0.0:3000` (HTTP API for players) and `0.0.0.0:11000` (TCP for ProcMan connections). These ports are currently
**hardcoded** in the source — there are no CLI flags or config options to change them.

> **TODO:** The API port (3000) and ProcMan listener port (11000) are hardcoded in `crates/tools/unhub/src/main.rs`.
> They should be configurable via `hub_config.ron` or CLI flags.

### Verify the Hub is running

```bash
curl http://localhost:3000/health
```

Expected response:

```json
{ "hub_version": "0.3.2", "uptime_seconds": 5 }
```

### `hub_config.ron` Reference

Generated on first run. You **must edit this** before ProcMan can connect.

```ron
(
    version: 1,
    official_server_keys: {},
    banned_uuids: [],
    allowed_procman_uuids: [],
)
```

| Field                   | Type                    | Description                                                                                                                                                                                                                                                                 |
| ----------------------- | ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `version`               | `u32`                   | Config schema version. Do not change manually.                                                                                                                                                                                                                              |
| `official_server_keys`  | `HashMap<Uuid, String>` | UUIDs of servers authorized to host "official" games, mapped to human-readable names. Not currently used for anything in v1 — reserved for future ranking/trust features.                                                                                                   |
| `banned_uuids`          | `HashSet<Uuid>`         | Player Installation UUIDs banned from Hub services (creating/joining rooms). Banned players can still play single-player and direct-IP multiplayer.                                                                                                                         |
| `allowed_procman_uuids` | `HashSet<Uuid>`         | **Critical.** Only ProcMan instances whose `installation_id` appears in this set will be accepted. If this is empty, **no ProcMan can connect**, which means no rooms can be created. You must add your ProcMan's UUID here after its first run generates one (see Step 3). |

### What happens if the Hub crashes or restarts

All room state is in-memory. If the Hub restarts, all active rooms are lost. ProcMan will detect the disconnect and
attempt to reconnect every 5 seconds. Dedicated server processes keep running (players already connected stay
connected), but no new rooms can be created or joined until the Hub is back and ProcMan reconnects.

`hub_config.ron` is persisted to disk (via atomic write: write to `.tmp`, rename). Bans and allowed UUIDs survive
restarts.

---

## Step 3: Start the Process Manager (`unprocman`)

ProcMan needs to find the `unhaunter_dedicated` binary. It looks at the path configured in `procman_config.ron`.

```bash
cd /path/to/your/deployment/procman
/path/to/unprocman
```

On first run, it generates a default `procman_config.ron` with a new random `installation_id` (UUID).

**First-run bootstrap problem:** ProcMan will immediately try to connect to the Hub, but the Hub will reject it because
its UUID isn't in `allowed_procman_uuids` yet. You need to:

1. Run `unprocman` once. Let it fail to connect. Note the `installation_id` it wrote to `procman_config.ron`.
2. Add that UUID to `hub_config.ron`'s `allowed_procman_uuids` list.
3. Restart the Hub (it reads config on startup, not dynamically).
4. Restart ProcMan.

> **TODO:** This bootstrap dance is painful. The Hub should either have a CLI command to add a ProcMan UUID, or should
> reload config on SIGHUP, or ProcMan's first-run output should explicitly print "Add this UUID to your Hub config:
> `<uuid>`" instead of just silently writing a config file.

**Expected output on successful connection:**

```
INFO unprocman: Connecting to Hub at localhost:11000...
INFO unprocman::hub_comm: Connected to Hub (version: 0.3.2)
INFO unprocman::manager: Spawning new idle server on port 12000
```

**If you see `Auth rejected by Hub`:** Your ProcMan's `installation_id` is not in the Hub's `allowed_procman_uuids`.

**If you see `Failed to connect to Hub: ... Retrying in 5s...`:** The Hub isn't running, or ProcMan's `hub_addr` is
wrong, or port 11000 is firewalled.

**If you see `No such file or directory` after "Spawning new idle server":** The `game_binary_path` in
`procman_config.ron` doesn't point to a valid `unhaunter_dedicated` binary.

### `procman_config.ron` Reference

```ron
(
    hub_addr: "localhost:11000",
    public_addr: "127.0.0.1",
    installation_id: "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    port_range: (12000, 12100),
    idle_pool_size: 1,
    game_binary_path: "./unhaunter_dedicated",
)
```

| Field              | Type         | Description                                                                                                                                                                                                                                                                                                                                                       |
| ------------------ | ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `hub_addr`         | `String`     | Address of the Hub's ProcMan TCP listener. For local dev: `"localhost:11000"`. For remote: `"hub.example.com:11000"`. This is the TCP port (not the HTTP API port).                                                                                                                                                                                               |
| `public_addr`      | `String`     | **The IP or domain that players will use to connect to game servers on this machine.** This is what gets sent to players via the Hub. If this is wrong, players will get a server address they can't reach. For local dev: `"127.0.0.1"`. For production: your server's public IP or domain.                                                                      |
| `installation_id`  | `Uuid`       | Auto-generated on first run. This must match an entry in the Hub's `allowed_procman_uuids`. Do not change it after registering it with the Hub.                                                                                                                                                                                                                   |
| `port_range`       | `(u16, u16)` | Range of TCP ports to assign to dedicated server processes. Each server gets one port. The range must not overlap with the Hub's ProcMan listener port (default 11000). Default: `(12000, 12100)`, giving you up to 100 simultaneous rooms. **These ports must be open for inbound TCP traffic from the internet** — this is how players connect to game servers. |
| `idle_pool_size`   | `usize`      | Number of pre-spawned idle `unhaunter_dedicated` processes to keep ready. When one gets assigned a room, ProcMan spawns a replacement. Higher values = faster room creation but more RAM/CPU used. Default: `1`. Each idle server costs ~50-100MB RAM and ~10% CPU on a 2-vCore machine (Bevy scheduler overhead).                                                |
| `game_binary_path` | `String`     | Path to the `unhaunter_dedicated` executable. Relative to ProcMan's working directory. The binary must have the `assets/` directory available (ProcMan attempts to set `CARGO_MANIFEST_DIR` to the workspace root for dev builds, but this is fragile and won't work for deployed binaries).                                                                      |

### Assets directory problem

`unhaunter_dedicated` is a Bevy application that needs the `assets/` folder at runtime (it loads map files). In a dev
build, Bevy finds assets via `CARGO_MANIFEST_DIR`. In a deployed build, Bevy looks for an `assets/` directory relative
to the binary's working directory.

ProcMan spawns the dedicated server as a child process. Its working directory is ProcMan's working directory, not the
binary's location. So you must either:

1. Run ProcMan from the repository root (where `assets/` exists), or
2. Copy/symlink `assets/` next to wherever ProcMan runs from, or
3. Copy/symlink `assets/` to the same directory as the `unhaunter_dedicated` binary and set `game_binary_path`
   accordingly.

> **TODO:** This is fragile and undocumented in the code. The dedicated server should accept an `--assets-dir` flag, or
> ProcMan should set the working directory of spawned processes to the binary's parent directory, or the release
> packaging should bundle assets with the server binary.

---

## Step 4: Verify the Stack

At this point you should have three processes running:

1. `unhub` — listening on ports 3000 (HTTP) and 11000 (TCP)
2. `unprocman` — connected to hub, managing dedicated servers
3. At least one `unhaunter_dedicated` — spawned by ProcMan, idle, waiting for a room assignment

### Check health

```bash
curl http://localhost:3000/health
```

### Create a test room

```bash
curl -X POST http://localhost:3000/v1/rooms/create \
  -H "Content-Type: application/json" \
  -d '{"player_uuid": "00000000-0000-0000-0000-000000000001", "game_version": "0.3.2-dev"}'
```

Expected response (if everything works):

```json
{ "code": "K7WRP", "addr": "127.0.0.1:12000", "secret": "..." }
```

**If you get `{"error":"no_capacity","message":"No server capacity available for this game version."}`:**

- ProcMan is not connected to the Hub (check ProcMan's `hub_addr` and the Hub's `allowed_procman_uuids`).
- Or there are no idle servers (check ProcMan logs for spawn errors).
- Or the `game_version` in the request doesn't match what ProcMan reports. The version is currently **hardcoded** as
  `"0.3.2-dev"` in `crates/tools/unprocman/src/hub_comm.rs`. It must match the version the game client sends.

> **TODO:** The game version string `"0.3.2-dev"` is hardcoded in ProcMan's hub communication code. This should come
> from the Cargo package version or a config field. If clients and servers are built from different commits, version
> mismatch silently prevents room creation with a misleading "no capacity" error.

**If you get `{"error":"timeout","message":"Timed out waiting for server allocation."}`:**

- ProcMan received the request but the dedicated server didn't start in time (5 seconds). Check ProcMan logs for errors
  spawning the binary.

### Join with a game client

```bash
cargo run --bin unhaunter_game -- --hub-url http://localhost:3000
```

In the main menu, navigate to "Play Online", then "Create Room" or "Join Room".

> **TODO:** The `--hub-url` flag overrides the default Hub URL. The default is configured in `assets/config/client.ron`
> but this file may not exist in the repository yet. The behavior when neither the flag nor the config file is present
> is unclear and may result in a silent failure to connect.

---

## Step 5: Production Deployment

### Network Requirements

| Port                       | Protocol   | Who connects | Direction                        | Purpose                                         |
| -------------------------- | ---------- | ------------ | -------------------------------- | ----------------------------------------------- |
| 3000                       | TCP (HTTP) | Game clients | Inbound to Hub                   | REST API (create/join rooms, health checks)     |
| 11000                      | TCP        | ProcMan      | **Outbound from ProcMan** to Hub | ProcMan ↔ Hub orchestration. ProcMan initiates. |
| 12000–12100 (configurable) | TCP        | Game clients | Inbound to ProcMan machine       | Direct game traffic. Each room uses one port.   |

- The Hub machine needs ports **3000** and **11000** open for inbound.
- The ProcMan/game-server machine needs the game **port range** open for inbound. ProcMan itself needs no inbound ports
  (it connects out to the Hub).
- If Hub and ProcMan are on different machines, ProcMan needs outbound access to Hub's port 11000.
- If Hub and ProcMan are on the **same** machine, the game port range must not include port 11000 (conflict with the
  Hub's ProcMan listener). The default config uses 12000–12100, which avoids this.

### TLS / HTTPS

The Hub's HTTP API (port 3000) should be behind a reverse proxy (e.g., Nginx, Caddy) with TLS for production. WASM
clients (future) will require HTTPS. The ProcMan TCP connection (port 11000) is **not encrypted** — it uses plaintext
JSONL over TCP.

> **TODO:** The ProcMan ↔ Hub connection has no encryption or authentication beyond UUID matching. On a public network,
> this is a security concern. TLS for the ProcMan link, or at minimum a shared secret, should be added before any
> serious production deployment.

### Systemd

> **TODO:** No systemd unit files are provided. Below is a rough template — it has not been tested.

```ini
# /etc/systemd/system/unhub.service
[Unit]
Description=Unhaunter Hub
After=network.target

[Service]
Type=simple
User=unhaunter
WorkingDirectory=/opt/unhaunter/hub
ExecStart=/opt/unhaunter/bin/unhub
Restart=always
RestartSec=5
Environment=RUST_LOG=unhub=info

[Install]
WantedBy=multi-user.target
```

```ini
# /etc/systemd/system/unprocman.service
[Unit]
Description=Unhaunter Process Manager
After=network.target unhub.service

[Service]
Type=simple
User=unhaunter
WorkingDirectory=/opt/unhaunter/procman
ExecStart=/opt/unhaunter/bin/unprocman
Restart=always
RestartSec=5
Environment=RUST_LOG=unprocman=info

[Install]
WantedBy=multi-user.target
```

Note: `unhaunter_dedicated` is managed by ProcMan, not by systemd. Do not create a service for it.

### Monitoring

- **Health endpoint:** `GET /health` on port 3000 returns `{"hub_version":"...","uptime_seconds":...}`. Use this for
  uptime monitoring.
- **Logs:** Both `unhub` and `unprocman` log to stderr via `tracing`. Control verbosity with the `RUST_LOG` environment
  variable (e.g., `RUST_LOG=unhub=debug` for verbose output).
- **ProcMan child output:** ProcMan captures stdout/stderr from dedicated servers and re-logs them with a `[Port NNNNN]`
  prefix, detecting log levels from Bevy's format.

> **TODO:** There are no metrics, no Prometheus endpoint, no structured log output (JSON). For a production deployment
> with multiple ProcMans, observability is effectively nonexistent beyond reading log lines.

### Backup

The only persistent state is `hub_config.ron` (bans, allowed ProcMans) and `procman_config.ron` (installation UUID). All
room state is in-memory and ephemeral. Back up the `.ron` files.

### Upgrades

> **TODO:** There is no documented upgrade procedure. The Hub and ProcMan have no graceful shutdown signal handling.
> Restarting the Hub drops all in-memory room state. Restarting ProcMan kills all dedicated server child processes. A
> zero-downtime upgrade path (drain rooms, restart, re-register) does not exist.

---

## Telling Players to Use Your Hub

Players override the default Hub URL with a CLI flag:

```bash
unhaunter_game --hub-url https://hub.your-community.example.com
```

> **TODO:** There is no in-game UI for entering a custom Hub URL. Players must use the CLI flag. The design proposal
> mentions an `assets/config/client.ron` with a "Universe" concept, but the client-side implementation status of this is
> unclear.

---

## Common Problems

| Symptom                                                        | Likely cause                                                     | Fix                                                                |
| -------------------------------------------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------ |
| ProcMan logs `Auth rejected by Hub`                            | ProcMan's `installation_id` not in Hub's `allowed_procman_uuids` | Add the UUID to `hub_config.ron`, restart Hub                      |
| ProcMan logs `Failed to connect to Hub: Connection refused`    | Hub isn't running, or `hub_addr` is wrong                        | Start the Hub, check the address and port                          |
| ProcMan logs `No such file or directory` when spawning servers | `game_binary_path` is wrong                                      | Fix the path in `procman_config.ron`                               |
| ProcMan logs `No free ports in range`                          | All ports in `port_range` are in use                             | Increase the range, or wait for rooms to close                     |
| `create_room` returns `no_capacity`                            | No ProcMan connected, or no idle servers, or version mismatch    | Check ProcMan connection, check logs, check version string         |
| `create_room` returns `timeout`                                | Dedicated server failed to start within 5s                       | Check ProcMan logs for spawn errors                                |
| `join_room` returns `code_not_found`                           | Typo, or room timed out (5 min idle), or Hub restarted           | Verify room code, create a new room                                |
| Players can't connect to game server                           | Game port range is firewalled                                    | Open the port range for inbound TCP                                |
| Game client shows no Hub connection indicator                  |                                                                  | This is a known missing feature (see CUJ inspection)               |
| Dedicated server crashes on startup                            | Missing `assets/` directory                                      | Ensure assets are available (see "Assets directory problem" above) |

---

## Known Limitations

This is an early implementation. Major gaps include:

- **No prebuilt binaries or containers.** You must build from source.
- **Hardcoded ports.** Hub API (3000) and ProcMan listener (11000) are not configurable.
- **Hardcoded game version.** ProcMan reports `"0.3.2-dev"` regardless of actual binary version.
- **No encryption on ProcMan ↔ Hub link.**
- **No graceful shutdown or drain.** Restarting any component disrupts active games.
- **No room capacity limit enforcement at the Hub level.** The Hub doesn't check player count before directing a join —
  the dedicated server rejects excess players, but the error message to the player is poor.
- **No auto-reconnect for game clients.** If a player disconnects, they must manually rejoin.
- **CPU overhead.** Each idle dedicated server consumes ~10% CPU on a 2-vCore machine due to Bevy's multithreaded
  scheduler running even when idle.
- **No player-facing error messages.** Hub errors (room not found, banned, etc.) are logged to the terminal but not
  displayed in the game UI.
