# Grand Master Plan: Multi-Version Dedicated Server Infrastructure

- **Status:** Design — Not Yet Implemented
- **Date:** April 2026

---

## 1. The Problem (Summary)

The current deployment is "State 0": one binary, one `assets/`, Ansible overwrites everything, `systemd` restarts. This
is fine for development but breaks when the game has public players.

Once players are in the wild on different client versions, they have different `bevy_replicon` protocol hashes. A client
and server with mismatched hashes refuse to connect — this is enforced by `bevy_replicon` at the network layer, not by
application logic. There is no workaround.

The goals are:

1. Never split compatible players across multiple server instances unnecessarily.
2. Keep incompatible players able to play on the server for their version for as long as the VPS can afford it.
3. Proactively inform players in the Main Menu when their version is obsolete so they don't hit a confusing timeout.
4. Keep the CPU cost on a 2-core VPS budget (roughly one idle server per active protocol hash, maximum ~6 at any time).

---

## 2. The Core Axiom: Protocol Hash as Compatibility Key

`bevy_replicon` generates a deterministic `u64` "protocol hash" from all registered replicated types (via
`app.replicate::<T>()`). **Two builds are network-compatible if and only if their protocol hashes are identical.**

Semantic version strings are meaningless for routing. `0.4.0` and `0.4.1` may or may not be compatible — only the
protocol hash knows.

**The routing key is `(channel, protocol_hash)`**, never `(channel, semver)`.

**When does the hash change?**

It changes when you add, remove, or rename a type passed to `app.replicate::<T>()`, add/remove/rename a field on a
replicated component, or change the registration order.

It does **not** change when you fix a non-networked bug, change assets, update systems that don't touch replicated
types, or do server-only logic changes.

---

## 3. The Four Channels

All earlier channel proposals (including `oldstable`, `oldbeta`, `oldstable`) are **withdrawn**. The system has exactly
four channels:

| Channel  | Who builds it       | How it arrives on VPS           | History |
| -------- | ------------------- | ------------------------------- | ------- |
| `stable` | GitHub Actions (CI) | Ansible fetches from GitHub CDN | Yes     |
| `beta`   | GitHub Actions (CI) | Ansible fetches from GitHub CDN | Yes     |
| `alpha`  | GitHub Actions (CI) | Ansible fetches `alpha-latest`  | No      |
| `dev`    | Developer's machine | Ansible `rsync` delta           | No      |

**`alpha` and `dev` are volatile targets**: there is exactly one copy on disk, always. Deploying a new one overwrites
the old one. Players on the old version are disconnected. This is the definition of a pre-release testing channel and is
acceptable.

**`stable` and `beta` keep a history** governed by the retention policy in Section 6.

---

## 4. Channel Versioning Strategy

### 4.1 Stable and Beta

These follow the existing workflow. A developer tags a commit in Git:

- Stable: `v0.4.1` → CI triggers `release-main.yml`
- Beta: `v0.4.1-beta2` → CI triggers `release-beta.yml`

The tag is the version. `Cargo.toml` reflects the correct version string on that commit. No changes to the current
tagging workflow are needed.

### 4.2 Alpha

The `alpha` git branch has **one dedicated version-bump commit** at its base that sets the `Cargo.toml` version to
`0.X.Y-alpha` (e.g., `0.4.0-alpha`). Every push to the `alpha` branch triggers a new CI build. The version string stays
`0.4.0-alpha` until the branch is rebased onto a new base version.

**Why this works without merge conflicts:** The `dev` branch has `0.4.0-dev`. The `alpha` branch forks from `dev` and
adds the version-bump commit on top. When `dev` advances (e.g., features merged in), `alpha` rebases that one extra
commit on top of the new `dev`. The Cargo.toml version field changes from `dev` and the version-bump commit overwrites
it again cleanly. No conflicts arise because the alpha commit _always sits on top_.

**CI behavior:** A GitHub Actions workflow (`release-alpha.yml`) fires on any push to the `alpha` branch. It builds
`unhaunter_dedicated` + assets and forcibly updates a single pinned GitHub Release named `alpha-latest`. See Section 7.3
for the CI recipe.

### 4.3 Dev

`dev` is built by the developer on their local machine. `Cargo.toml` has `version = "0.4.0-dev"` (or whatever the
current cycle is). When a developer is ready to deploy a test build, they run the Ansible playbook and `rsync` syncs
only the changed bytes. No CI, no GitHub, no version tags.

---

## 5. The Library: Directory Structure on the VPS

```text
/opt/unhaunter/
  unhub                   (singleton infrastructure binary)
  unprocman               (singleton infrastructure binary)
  hub_config.ron
  procman_config.ron
  library/
    stable/
      0.3.5/
        unhaunter_dedicated
        assets/
      0.4.0/
        unhaunter_dedicated
        assets/
      0.4.1/
        unhaunter_dedicated
        assets/
    beta/
      0.4.2-beta1/
        unhaunter_dedicated
        assets/
      0.4.2-beta2/
        unhaunter_dedicated
        assets/
    alpha/
      unhaunter_dedicated
      assets/
    dev/
      unhaunter_dedicated
      assets/
```

Each version bundle is **self-contained**: one binary + one `assets/` directory. This prevents `v0.3.5` from
accidentally reading a `v0.4.1` tileset.

Assets are ~90 MB, the binary is ~136 MB. Even 10 full bundles on disk total just under 2.3 GB. A 20 GB VPS handles this
comfortably.

---

## 6. Library Retention Policy

The retention policy is enforced by a cleanup step in the Ansible playbook (or a manual script) that runs **after**
deployment. It never removes a version that is currently running.

### 6.1 Stable Retention

Keep the following bundles for `stable/`:

1. **Latest patch of the current minor version.** (e.g., `0.4.3` when that is current)
2. **Previous patch of the current minor version**, but only if it has a different protocol hash than the latest. If
   `0.4.2` and `0.4.3` share a hash, `0.4.2` is redundant — delete it.
3. **Latest patch of the previous minor version.** (e.g., keep `0.3.5`)

Everything older than the previous minor version is deleted.

**Example state when `0.4.3` is current:**

- Keep: `0.4.3` ✓
- Keep: `0.4.0` ✓ (different hash than `0.4.3`)
- Keep: `0.3.5` ✓ (previous minor, latest patch)
- Delete: `0.4.1`, `0.4.2` (same hash as `0.4.3`, or older than retained)
- Delete: `0.2.7` (two full minors behind)

**Extended compatibility mode (optional):** Keep _all_ patches of the current minor version that have unique protocol
hashes. This ensures every player who has downloaded the current minor release can still play. This is the maximum
supportable set.

### 6.2 Beta Retention

Keep up to **3 most recent beta bundles** where each has a distinct protocol hash. If `beta1` and `beta2` share a hash,
delete `beta1`.

### 6.3 Alpha and Dev

No retention. There is exactly one copy. It is overwritten on each deploy.

---

## 7. Artifact Delivery: How Binaries Get to the VPS

A **single Ansible playbook** (`deploy/multiplayer.yml`) handles everything. The developer runs it from their laptop.
There is no cron job, no VPS-initiated pull, no ghost in the machine.

```bash
ansible-playbook deploy/multiplayer.yml -i "YOUR_VPS_IP," -u USER -e "domain=hub.yourdomain.com"
```

### 7.1 Infrastructure Binaries (unhub, unprocman)

Same as current behavior: `rsync` from the local `target/release/` to `/opt/unhaunter/`. These are singleton binaries
with no versioned history.

> **⚠ CONFLICT NOTED:** The current Ansible playbook sets `ticket_hmac_secret = {{ procman_uuid }}` in
> `procman_config.ron`. This makes the HMAC secret predictable (equal to a publicly derivable UUID). The
> ticket_hmac_secret should be a cryptographically random 64-character hex string. This should be fixed independently of
> the multi-version work.

### 7.2 Stable and Beta

Ansible tasks query the GitHub Releases API, identify the correct release assets, and instruct the **VPS** to `wget`
them from GitHub's CDN:

```text
GET https://api.github.com/repos/deavid/unhaunter/releases
```

The playbook selects the appropriate releases per channel, downloads the
`unhaunter-<version>-server-linux-x86_64.tar.gz` archive to the VPS, extracts it into
`/opt/unhaunter/library/<channel>/<version>/`, and runs the retention cleanup step (Section 6).

The download happens server-to-GitHub, **not laptop-to-server**, so no developer bandwidth is consumed for these
channels.

### 7.3 Alpha

Ansible fetches the `alpha-latest` rolling pre-release from GitHub and extracts it directly to
`/opt/unhaunter/library/alpha/`, overwriting the previous copy.

**GitHub Releases rolling-release mechanics:** GitHub Releases are immutable by content but the `gh` CLI supports
forcibly deleting and recreating a release with the same tag. The `release-alpha.yml` CI workflow does this atomically:

```yaml
- name: Overwrite alpha-latest release
  env:
    GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  run: |
    gh release delete alpha-latest --yes --cleanup-tag 2>/dev/null || true
    gh release create alpha-latest \
      --prerelease \
      --title "Alpha Latest (rolling)" \
      --notes "Latest alpha build. Not for production." \
      releases/unhaunter-server-*
```

This release must **not** be marked `--latest`. It must never appear above stable releases in the GitHub UI. The URL
structure for the asset will be:
`https://github.com/deavid/unhaunter/releases/download/alpha-latest/unhaunter-server-linux-x86_64.tar.gz`

### 7.4 Dev

Ansible uses `rsync` to push only the changed bytes from the local `target/release/unhaunter_dedicated` and local
`assets/` directly to `/opt/unhaunter/library/dev/`. No GitHub involved.

---

## 8. Binary Self-Description: The `--print-info` Flag and Metadata Files

`unprocman` reads version metadata from **static text files** in each bundle directory, not by executing binaries. This
keeps `unprocman` simple, avoids security issues (no arbitrary execution on scan), and works identically for `dev`,
`alpha`, `stable`, and `beta` bundles.

### 8.1 The Metadata Files

Every valid bundle directory must contain two plain-text files:

- `VERSION` — contains the raw version string, e.g. `0.4.1`
- `PROTOCOL_HASH` — contains the `bevy_replicon` protocol hash (decimal `u64`), e.g. `8187382949123456789`

If either file is absent, `unprocman` **ignores the entire folder** and emits a warning to its log. This prevents
partially deployed bundles from being accidentally served.

### 8.2 Who Creates the Files: Ansible

These files are generated by the **Ansible playbook** immediately after uploading the binary to the VPS. Ansible
executes the binary using `--print-info` (once, on the VPS) and writes the output into the two files:

```bash
./unhaunter_dedicated --print-info
# → prints JSON to stdout
```

Ansible parses the JSON and writes:

```bash
echo "0.4.1" > VERSION
echo "8187382949123456789" > PROTOCOL_HASH
```

This approach is used for **all channels**, including `dev` (where `rsync` brings the binary, then Ansible runs it
locally on the VPS to generate the files). The files are not committed to Git and are not included in release archives.
They are always generated fresh on the VPS at deploy time.

### 8.3 The `--print-info` Flag (Binary Side)

The dedicated server binary must implement `--print-info`:

1. Does NOT start the full Bevy game loop.
2. Prints its metadata to stdout as JSON.
3. Exits immediately.

**Expected output:**

```json
{
  "version": "0.4.1",
  "channel": "stable",
  "protocol_hash": 8187382949123456789,
  "build_time_utc": "2026-04-25T14:30:00Z"
}
```

Fields:

- `version`: The `CARGO_PKG_VERSION` string.
- `channel`: One of `"stable"`, `"beta"`, `"alpha"`, `"dev"`. Baked in at build time.
- `protocol_hash`: The `bevy_replicon` protocol hash (`u64`).
- `build_time_utc`: ISO 8601 UTC timestamp, embedded at compile time via `build.rs`.

### 8.1 OPEN QUESTION: How to Extract the Protocol Hash Without a Full Boot

**This is the primary unresolved implementation question.**

The `bevy_replicon` protocol hash is computed at runtime after all `app.replicate::<T>()` calls complete. These calls
happen during plugin setup in `App::build()`, before the Bevy main loop starts. There is no compile-time mechanism to
extract this hash without running the Bevy application.

**The implication:** `--print-info` must actually run all plugin setup (call `App::build()` and register all plugins),
read the hash from `RepliconServer`/`RepliconClient` resources, print it, and `process::exit(0)` — all before the main
loop starts.

This means `--print-info` incurs a 2–5 second startup cost (Bevy asset loading can be skipped since replicon types are
registered without assets), and it will attempt to open display windows on a headless server unless headless mode is
properly enforced.

**Likely solution:** Add a dedicated early-exit path in `app_run()`: if the process was invoked with `--print-info`, run
only the plugin registration phase (using a minimal headless runner), extract the hash from the replicon state, print
it, and exit before the Bevy schedule starts.

This needs to be tested and confirmed. **Do not assume it will work without implementation experiment.** The person
implementing this should verify that:

- The Bevy `App` can be partially constructed (up to plugin setup) without starting the ECS schedule.
- The replicon hash is accessible after `App::build()` but before `App::run()`.

---

## 9. The procman_config.ron Extension

The current config has a single `game_binary_path: String`. This must be replaced with a library-aware pool
configuration.

**Proposed new schema** (final schema TBD during implementation):

```ron
(
    hub_addr: "127.0.0.1:11000",
    public_addr: "1.2.3.4",
    installation_id: "...",
    ticket_hmac_secret: "64-hex-chars",
    port_range: (12000, 12100),
    library_dir: "/opt/unhaunter/library",
    // Maximum number of dedicated server processes unprocman may run simultaneously
    // across all channels and all versions. This is a global guard against accidental
    // DoS of the VPS. There are no idle pools — all servers are strictly on-demand.
    max_total_instances: 8,
)
```

`unprocman` scans `library_dir` on startup and on `SIGHUP`. For each subdirectory found, it looks for `VERSION` and
`PROTOCOL_HASH` text files. Folders without both files are silently ignored (with a warning log). There are no
per-channel capacity targets — `max_total_instances` is the only global limit.

---

## 10. `unprocman` Library Scan and On-Demand Spawning

### 10.1 Library Scan

On startup and on SIGHUP (the reload signal), `unprocman`:

1. Recursively scans all subdirectories of `library_dir`.
2. For each directory, reads `VERSION` and `PROTOCOL_HASH` as plain text. If either file is absent, the directory is
   skipped with a warning log.
3. Builds an in-memory `BinaryInfo` table: `{path, version, protocol_hash}` for every valid bundle found.
4. Reports the full table to `unhub` on the next heartbeat.

`unprocman` does **not** call `--print-info` at scan time. It does **not** execute any binary during a scan.

### 10.2 No Superseding Rule

There is no active superseding or drain logic inside `unprocman`. Version management happens at the Ansible level:

- When a new version is deployed, Ansible runs `systemctl restart unprocman`.
- Restarting `unprocman` severs all `stdin` pipes to child Bevy processes.
- All running Bevy games detect `EOF` on `stdin` and immediately self-terminate (Shared Fate).
- Active players are disconnected. This is acceptable — "tough luck" for the rare case where a deploy lands during a
  live game.

This eliminates an entire category of complex state-machine logic in `unprocman`. Deployments are blunt restarts.

> **⚠ BLAST RADIUS NOTE:** Because `unprocman` is a singleton managing **all** channels on the VPS, any deploy that
> restarts `unprocman` — including a routine `dev` channel push — will **instantly kill all active players on `stable`
> and `beta`** as well. This is a known and accepted tradeoff at current scale. Be aware that pushing a quick local test
> build nukes the entire global server. Do not be surprised when it happens.

### 10.3 On-Demand Process Spawning

`unprocman` maintains **zero idle servers**. All servers are strictly on-demand:

1. `unhub` sends a `CreateRoom` request to `unprocman` with `{version, protocol_hash}`.
2. `unprocman` looks up the matching bundle path in its `BinaryInfo` table.
3. `unprocman` spawns the binary, pipes config JSON to `stdin`, waits for `{"type": "Ready", "port": ...}` on `stdout`.
4. `unprocman` replies to `unhub` with the connection details.
5. When the game ends and all players leave, the Bevy app calls `exit(0)`, freeing the port.

**Concurrency guard:** Only 1 spawn-in-progress is allowed at a time per `unprocman` node. If `unhub` sends a second
`CreateRoom` while a spawn is in progress, `unprocman` hard-rejects it immediately (no queue). The error propagates back
to the client. The client retries by clicking again a few seconds later.

**Spawn timeout:** If the Bevy app does not emit `{"type": "Ready"}` within 10 seconds, `unprocman` kills the process
and returns `CreateRoomFailed` to `unhub`.

### 10.4 Reload Signal

Ansible sends `SIGHUP` (or an equivalent admin API call — TBD) to `unprocman` after deploying new binaries. `unprocman`
performs a library rescan without restarting the process itself.

---

## 11. `unhub` Protocol Extensions

### 11.1 ProcManHello Extension

`unprocman` must report its full library to `unhub` during the handshake, not just the currently running versions.
`unhub` uses this to answer client pings with accurate upgrade recommendations, even for versions not yet running.

**New fields in `ProcManHello`:**

```rust
library: Vec<LibraryEntry>,
```

Where `LibraryEntry` is:

```rust
pub struct LibraryEntry {
    pub version: String,
    pub channel: String,
    pub protocol_hash: u64,
}
```

This replaces the current `game_versions: Vec<String>`. The heartbeat should similarly carry library state so unhub
stays in sync if binaries are added/removed between heartbeats.

### 11.2 CreateRoom Extension

Currently, `CreateRoom` carries `game_version: String`. This is used to match a procman that supports that version. In
the multi-version model, routing must be by `(channel, protocol_hash)`.

**New fields in `CreateRoom`:**

```rust
pub channel: String,
pub protocol_hash: u64,
```

`unhub` finds a procman with matching `(channel, protocol_hash)` capacity.

### 11.3 Ping Extension

The current `/v1/ping` only accepts `installation_id`. It must be extended to also carry the client's version, protocol
hash, and channel.

**New `PingRequest`:**

```rust
pub struct PingRequest {
    pub installation_id: Uuid,
    pub version: String,           // e.g. "0.4.0"
    pub protocol_hash: u64,        // from bevy_replicon
    pub channel: String,           // "stable", "beta", "alpha", "dev"
}
```

**New `PingResponse`:**

```rust
pub struct PingResponse {
    pub ok: bool,
    pub online_players_estimate: usize,
    pub multiplayer_status: MultiplayerStatus,
    pub upgrade_version: Option<String>,   // version to upgrade to
    pub upgrade_channel: Option<String>,   // channel to upgrade to
}

pub enum MultiplayerStatus {
    UpToDate,
    UpdateAvailable,       // same hash, newer semver
    UpdateRecommended,     // different hash, old server still running
    Unsupported,           // no server for this hash+channel
}
```

---

## 12. `unhub` Routing Logic

### 12.1 For `stable` and `beta`

When a client creates a room:

1. Look at all connected procmans.
2. Filter to those that have at least one idle instance with matching `(channel, protocol_hash)`.
3. Among matching procmans, prefer the one whose running binary has the highest semver.
4. Route the `CreateRoom` request there.

### 12.2 For `alpha` and `dev`

One procman. One binary. One idle instance. Route directly. If the server's hash doesn't match the client's,
`bevy_replicon` will reject the connection at the TCP handshake level. `unhub` does not need to pre-filter this. The
resulting failure will be reflected in the next ping response as `Unsupported`.

### 12.3 Ping Status Logic

When `unhub` receives a `PingRequest`:

```text
client_hash = request.protocol_hash
client_channel = request.channel
client_version = semver_parse(request.version)

best_same_hash = highest semver in library for (client_channel, client_hash)
best_stable    = highest semver in library for ("stable", any hash)
has_server     = any connected procman has this (client_channel, client_hash) in its library

if client_version >= best_same_hash AND client_channel == "stable":
    → UpToDate

if best_same_hash > client_version and same hash as client:
    → UpdateAvailable (they can upgrade without losing server access)

if best_stable > client_version (by semver pre-release rules):
    → recommend upgrade to stable

if has_server:
    → UpdateRecommended (server still runs for their hash; new hash exists)
else:
    → Unsupported
```

**Channel upgrade funnel (one-way):** Upgrade recommendations only move toward `stable`. A client on `stable` is never
recommended to try `beta`. A client on `beta` is recommended `stable` if a stable release exists with higher semver. A
client on `alpha` or `dev` is always recommended `beta` or `stable` unless they are both on the exact matching hash.

---

## 13. Client-Side Changes

### 13.1 Read Protocol Hash at Runtime

During `Startup` (after all plugins have registered their types), the client must read the `bevy_replicon` protocol hash
and store it in a resource for use during the hub ping and room creation requests.

### 13.2 Channel Constant

Each binary must know its own channel at compile time. A build-time constant `CHANNEL` in `unhub-client` crate (or
equivalent) carries the value:

- `"stable"` — baked into release builds from CI tagged with `v*` (no pre-release suffix)
- `"beta"` — baked into release builds from CI tagged with `v*-beta*`
- `"alpha"` — baked into builds from the `alpha` branch CI
- `"dev"` — baked into all other local builds

This can be a `const &str` set via a build-time environment variable in `build.rs`, or it can be inferred from the
`CARGO_PKG_VERSION` suffix:

- ends with `-dev` → `"dev"`
- ends with `-alpha` → `"alpha"`
- contains `-beta` → `"beta"`
- otherwise → `"stable"`

Inferring from the version suffix is simpler, requires no extra build machinery, and is **the recommended approach**
given David's commitment to no CI version injection.

### 13.3 Include Channel and Hash in All Hub Requests

- `PingRequest`: include `version`, `protocol_hash`, `channel`.
- `CreateRoomRequest`: include `protocol_hash`, `channel`.
- `JoinRoomRequest`: no change needed (hash mismatch is detected at the server level).

### 13.4 Multiplayer UI States

The client Main Menu reacts to `MultiplayerStatus` in `PingResponse`:

| Status              | Multiplayer Button | Banner                                                                               |
| ------------------- | ------------------ | ------------------------------------------------------------------------------------ |
| `UpToDate`          | Fully enabled      | None                                                                                 |
| `UpdateAvailable`   | Fully enabled      | Yellow subtle text: "Update available: v0.4.1"                                       |
| `UpdateRecommended` | Fully enabled      | Yellow prominent banner: "Major update available. Update to play with more players." |
| `Unsupported`       | **Disabled**       | Red banner: "Version unsupported. Download the latest version to play online."       |

For `Unsupported` on `alpha`/`dev` channels: direct users to the `beta` or `stable` channel download page, not to
another dev build.

---

## 14. GitHub Actions CI Changes

### 14.1 New Justfile Recipes Required

The current Justfile has no recipe for `unhaunter_dedicated`. All existing recipes package `unhaunter_game`. **This is a
critical gap**: stable and beta workflows call `just package-all` which does NOT include the dedicated server binary.

The server binary is **Linux-only** — no Windows or WASM builds are needed for the dedicated server.

New recipes to add, following existing Justfile conventions:

```just
# Build the dedicated server binary (Linux only)
build-server:
    echo "Building server release..."
    cargo build --release --target x86_64-unknown-linux-gnu --bin unhaunter_dedicated
    echo "Server build complete."

# Package the server bundle (binary + assets) into a tarball
# Does NOT depend on package-common: the server needs no screenshots, README,
# CHANGELOG, or upscaled assets. Only the binary and raw assets/ directory.
package-server: ensure-dist-dir build-server
    echo "Packaging server artifact for {{_version}}..."
    rm -rf {{_dist_dir}}/server
    mkdir -p {{_dist_dir}}/server
    cp {{_target_dir}}/x86_64-unknown-linux-gnu/release/unhaunter_dedicated \
       {{_dist_dir}}/server/unhaunter_dedicated
    cp -r {{_assets_dir}} {{_dist_dir}}/server/assets
    unlink {{_releases_dir}}/unhaunter-{{_version}}-server-linux-x86_64.tar.gz || true
    tar -czvf {{_releases_dir}}/unhaunter-{{_version}}-server-linux-x86_64.tar.gz \
        -C {{_dist_dir}}/server .
    echo "Server package created: {{_releases_dir}}/unhaunter-{{_version}}-server-linux-x86_64.tar.gz"
```

The server bundle copies the full `assets/` directory. Future optimization: strip image files (`.png`, `.jpg`) since the
dedicated server never renders them — this could halve the archive size. See Q5 in Section 17 before implementing that
optimization.

**`package-all` must be extended** to include `package-server`:

```just
# Create all release packages (now includes dedicated server)
package-all: package-linux package-windows package-wasm package-server
```

### 14.2 release-main.yml and release-beta.yml

No structural changes needed once `package-all` includes `package-server`. The release artifact name will be
`unhaunter-<version>-server-linux-x86_64.tar.gz`. Ansible will know to look for an asset matching this naming pattern.

### 14.3 New: release-alpha.yml

A new workflow targeting the `alpha` branch:

```yaml
name: Release Alpha (Rolling)

on:
  push:
    branches:
      - alpha

permissions:
  contents: write

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Update APT & Install Build Dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev

      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install just
        uses: extractions/setup-just@v3

      - name: Build Server Package
        run: just package-server

      - name: Overwrite alpha-latest rolling release
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          gh release delete alpha-latest --yes --cleanup-tag 2>/dev/null || true
          gh release create alpha-latest \
            --prerelease \
            --title "Alpha Latest (rolling)" \
            --notes "Latest alpha build from the alpha branch. Volatile. May break." \
            releases/unhaunter-*-server-linux-x86_64.tar.gz

      # Set the publish date 10 years in the past so this release never appears
      # at the top of the GitHub Releases page and does not clutter the public
      # "latest release" UI. Requires the GitHub CLI >= 2.x.
      gh release edit alpha-latest --latest=false
```

**Important:** The `--cleanup-tag` flag deletes both the release AND the git tag `alpha-latest`. Then
`gh release create` recreates both. This is intentional — the tag always points to the latest alpha commit. The download
URL will be stable:
`https://github.com/deavid/unhaunter/releases/download/alpha-latest/unhaunter-<version>-server-linux-x86_64.tar.gz`

The filename will contain the version string (e.g. `0.4.0-alpha`), which the Ansible playbook can discover by listing
the release assets via the GitHub API.

---

## 15. Ansible Playbook Changes

### 15.1 One Script, All Channels

The single `deploy/multiplayer.yml` playbook handles all four channels. There are no separate "deploy dev", "deploy
stable" scripts. The same command deploys everything:

```bash
ansible-playbook deploy/multiplayer.yml -i "YOUR_VPS_IP," -u USER -e "domain=hub.yourdomain.com"
```

**What happens in one run:**

1. Deploys infrastructure (`unhub`, `unprocman`) via rsync from local.
2. Deploys `dev` channel bundle via rsync from local.
3. Fetches `alpha-latest` from GitHub and updates `/opt/unhaunter/library/alpha/`.
4. Fetches the latest `stable` release from GitHub and adds it to `/opt/unhaunter/library/stable/<version>/` (if not
   already present).
5. Fetches the latest `beta` pre-release from GitHub and adds it to `/opt/unhaunter/library/beta/<version>/` (if not
   already present).
6. Runs the retention cleanup to prune old versions (per policy in Section 6).
7. Sends SIGHUP to `unprocman` to trigger library rescan.
8. Restarts `unhub` and `unprocman` systemd services only if their binaries changed.

### 15.2 GitHub API Usage

The playbook uses the `uri` module to query the GitHub API without authentication (public repo, public releases):

```yaml
- name: Fetch GitHub releases
  uri:
    url: "https://api.github.com/repos/deavid/unhaunter/releases"
    return_content: yes
  register: github_releases
```

From the response, the playbook filters by `prerelease: false` for stable and by tag name pattern for beta, then
downloads the matching `unhaunter-*-server-linux-x86_64.tar.gz` asset.

### 15.3 Idempotency

All operations are idempotent:

- If a `stable/<version>/` directory already exists, skip the download.
- `rsync` for dev is inherently delta-based.
- The `alpha-latest` overwrite always succeeds.
- Retention cleanup uses version metadata (not deletion by guess) to determine what to remove.

### 15.4 Updated procman_config.ron Template

The current template has `game_binary_path: "./bin/unhaunter_dedicated"`. Once `unprocman` supports the library scan,
this must be replaced with the schema from Section 9.

---

## 16. Implementation Sequence

The work must be executed in a specific order because each layer depends on the one below. An implementation in the
wrong order results in non-functional intermediate states.

### Phase 1 — Binary Self-Description (No Protocol Changes)

**Goal:** `--print-info` works and the dedicated server knows its own channel.

**Protocol hash extraction must be done first** — this is the primary implementation unknown and should be validated by
the developer hands-on before any other Phase 1 work starts. The outcome (what actually works) determines how
`--print-info` is implemented.

- [ ] **[First]** Investigate and implement protocol hash extraction. Recommended approach: add an early-exit path in
      the dedicated binary's `main()` that boots Bevy to plugin-setup completion (registers all plugins, runs
      `App::build()`), reads the hash from the replicon resource, prints it, and calls `process::exit(0)` before the ECS
      schedule starts. Validate that the hash is stable across repeated runs. Validate it matches what `bevy_replicon`
      uses for network rejection.
- [ ] Add `--print-info` flag to `dedicated.rs` using the extraction approach validated above.
- [ ] Infer channel from `CARGO_PKG_VERSION` suffix in `unhaunter_dedicated` and include it in the `--print-info`
      output.
- [ ] Add `package-server` recipe to Justfile (Linux-only; see Section 14.1 for the exact recipe).
- [ ] Update `package-all` to include `package-server`.

### Phase 2 — Ansible Library Structure (No Code Changes)

**Goal:** The VPS library directory works and the Ansible playbook can populate it.

- [ ] Add library directory creation tasks to Ansible playbook.
- [ ] Add `dev` channel rsync to `library/dev/` in Ansible.
- [ ] Add GitHub API fetch task for stable releases to Ansible.
- [ ] Add GitHub API fetch task for beta pre-releases to Ansible.
- [ ] Add GitHub API fetch task for `alpha-latest` to Ansible.
- [ ] Add retention cleanup tasks to Ansible.
- [ ] Write and test the `release-alpha.yml` CI workflow.
- [ ] Verify `release-main.yml` and `release-beta.yml` include server bundle in artifacts.

### Phase 3 — unprocman Library Scanning

**Goal:** `unprocman` reads the library and manages multi-version pools.

- [ ] Extend `ProcManConfig` with `library_dir` and `max_total_instances` fields. Remove `game_binary_path`.
- [ ] Update `procman_config.ron` template in `deploy/multiplayer.yml`: remove `game_binary_path` and `idle_pool_size`,
      add `library_dir: "/opt/unhaunter/library"` and `max_total_instances: 8`. See Section 9 for the target schema.
- [ ] Implement library scan on startup: recursively enumerate subdirs of `library_dir`, read `VERSION` and
      `PROTOCOL_HASH` text files, build in-memory `BinaryInfo` table. Skip and warn on missing files.
- [ ] Implement SIGHUP handler to trigger rescan without restarting the process.
- [ ] Implement on-demand spawning: spawn only when `unhub` sends `CreateRoom`. Zero idle servers.
- [ ] Implement concurrency guard: hard-reject new `CreateRoom` if a spawn is already in-progress.
- [ ] Implement spawn timeout (10s): kill process and return `CreateRoomFailed` if not ready in time.
- [ ] Implement hard 2-hour TTL: `SIGKILL` any child process older than 2 hours.
- [ ] Extend `ProcManHello` to carry `library: Vec<LibraryEntry>`.
- [ ] Update heartbeat to carry current library state.

### Phase 4 — unhub Protocol Extensions

**Goal:** `unhub` routes by hash and answers version-aware pings.

- [ ] Update `ProcManHello` handler to parse `library: Vec<LibraryEntry>`.
- [ ] Store library manifest per procman in `HubState`.
- [ ] Update `CreateRoom` routing to use `(channel, protocol_hash)` instead of `game_version: String`.
- [ ] Extend `PingRequest` / `PingResponse` structs in `unhub-client::protocol`.
- [ ] Implement `evaluate_client_status()` in `unhub::api::ping` using rules from Section 12.3.

### Phase 5 — Client Integration

**Goal:** Client reads its hash, sends it to hub, reacts to ping responses.

- [ ] In `unreplicon-plugin` (or `unhub-plugin`): read `bevy_replicon` protocol hash during `Startup` from the
      `RepliconServer`/`RepliconClient` resource and store in a local resource.
- [ ] Infer `channel` from `CARGO_PKG_VERSION` in the client build.
- [ ] Update `PingRequest` construction to include `version`, `protocol_hash`, `channel`.
- [ ] Update `CreateRoomRequest` to include `protocol_hash`, `channel`.
- [ ] Add UI reaction to `MultiplayerStatus` in the Main Menu plugin.

### Phase 6 — HOSTING.md and Documentation Update

- [ ] Update `HOSTING.md` to reflect the library directory, new Ansible command, and updated manual bootstrap steps.
- [ ] Update or archive `05` and `06` (leave as-is with a header pointing to `07`).

---

## 17. Open Questions Requiring Resolution Before Implementation

These are unresolved issues discovered during codebase analysis that were not addressed in the design conversation. **Do
not implement any phase without first confirming the team's preferred answer for each question in that phase.**

### Q1. Protocol Hash Extraction Mechanism (Phase 1 blocker)

The `bevy_replicon` protocol hash is computed at runtime. `--print-info` needs it, and is called **by Ansible** once per
bundle at deploy time (not by `unprocman` at scan time — `unprocman` only reads the resulting text files).

The recommended approach:

- **(A)** Manually trigger Bevy plugin setup without calling `App::run()`. In the `--print-info` branch, construct the
  `App`, add all plugins, then call `app.finish()` followed by `app.cleanup()`. This triggers the `finish()` hook on all
  plugins and runs setup systems, which is enough for `bevy_replicon` to register all types. At that point, extract the
  hash from `app.world()` directly, print it, and `process::exit(0)`. Cost: **milliseconds**, not seconds, because no
  asset loading or ECS schedule is started.
- **(B)** Pre-compute the hash at compile time using a `build.rs` that mirrors all `app.replicate::<T>()` registrations.
  Fragile — would need to be kept in sync with game code changes and would break silently on mismatch.

**Recommendation: Option A** using `app.finish()` + `app.cleanup()`. Note: `build.rs` is explicitly undesirable in this
codebase (causes unnecessary rebuilds). The exact resource path to the protocol hash in `bevy_replicon` must be
confirmed during implementation — verify against the `bevy_replicon` version in use.

### Q2. SIGHUP vs Admin API for unprocman Reload (Phase 3)

SIGHUP is a Unix signal, easy for Ansible (`kill -HUP $(systemctl show -p MainPID ...)`) but not portable to Windows. An
HTTP admin endpoint on `unprocman` is more explicit but requires adding an HTTP server.

**Recommendation:** Use SIGHUP for now. The server is Linux-only.

### Q3. ticket_hmac_secret Security Fix — **Resolved**

~~The current Ansible template sets `ticket_hmac_secret = {{ procman_uuid }}`. This made the HMAC secret identical to
the `procman_uuid`, which is deterministic and thus predictable.~~

**Fixed.** `deploy/multiplayer.yml` now generates a cryptographically random 32-byte hex secret via
`openssl rand -hex 32` on first deploy, persists it at `/opt/unhaunter/secrets/procman_hmac_secret` (mode `0600`), and
slurps it on subsequent runs without regenerating. The secret is rotatable via `-e rotate_hmac_secret=true`. This
blocker is cleared — the multi-version plan may proceed to deployment.

### Q4. procman_config.ron Migration Path (Phase 3)

The current `procman_config.ron` on the live VPS uses `game_binary_path`. The new schema uses `library_dir`. Ansible
must handle migrating existing config files without breaking running instances. One approach: Ansible checks whether
`game_binary_path` is present and migrates to the new schema in-place before signaling `unprocman`.

### Q5. Justfile build-server: Image Exclusion from assets/ — **Do Not Implement**

**Decision: leave the full `assets/` directory in the server bundle. Do not strip images.**

The game uses `bevy_asset_loader` and Tiled map plugins. If a `.tmx` or `.tsx` file references a `.png` that is
physically absent from disk, the `AssetServer` marks it as `Failed`. `bevy_asset_loader` will then sit in its `Loading`
state **forever**, waiting for all dependencies to resolve. The server will boot, hang indefinitely, and hit the
10-second `unprocman` spawn timeout every time.

The 90 MB of assets is not a problem on a 20 GB VPS. The risk of breaking headless boot far outweighs the disk saving.

---

## 18. What Does NOT Change

For clarity, the following things are **explicitly out of scope** and should not be modified as part of this work:

- `unhub` and `unprocman` multi-versioning. They remain single-version singletons. Their deployment (Section 7.1) does
  not change structurally. Room state resets on their restart — this is accepted as-is.
- The Caddy reverse proxy configuration.
- The `bevy_renet` / `renet` transport layer.
- Ticket authentication mechanism (HMAC structure stays the same; Q3 is a config/values fix, not a mechanism change).
- The Windows and WASM client build pipelines.
- Game logic, systems, or Bevy plugin architecture.

---

## 19. Summary of Artifacts to Create

| Artifact                                              | Type                           | Status      |
| ----------------------------------------------------- | ------------------------------ | ----------- |
| `deploy/multiplayer.yml`                              | Ansible playbook (rewritten)   | Not started |
| `.github/workflows/release-alpha.yml`                 | GitHub Actions workflow        | Not started |
| `Justfile` (add `build-server`, `package-server`)     | Justfile additions             | Not started |
| `unhaunter/src/bin/dedicated.rs` (add `--print-info`) | Rust code                      | Not started |
| `crates/tools/unprocman/src/config.rs`                | Rust config schema             | Not started |
| `crates/tools/unprocman/src/manager.rs`               | Rust logic                     | Not started |
| `crates/unhub-client/src/protocol.rs`                 | Rust protocol types            | Not started |
| `crates/tools/unhub/src/api.rs` (ping endpoint)       | Rust endpoint                  | Not started |
| `crates/tools/unhub/src/procman.rs`                   | Rust procman handler           | Not started |
| `crates/tools/unhub/src/state.rs`                     | Rust hub state                 | Not started |
| Client ping + channel constant                        | Rust (unreplicon/unhub plugin) | Not started |
| Client UI (`MultiplayerStatus` banner)                | Rust (UI plugin)               | Not started |
| `HOSTING.md` update                                   | Documentation                  | Not started |
