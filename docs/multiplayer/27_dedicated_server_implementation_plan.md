# Plan 27: Dedicated Server — Implementation Plan (Phase 1)

- **Date:** 2026-02-12
- **Status:** Ready for Implementation (reviewed & decisions finalized)
- **Depends on:** Plan 26 (Thin Server Analysis — complete)

---

## Goal

Deliver a working headless dedicated server binary that can host a single multiplayer mission. Players connect with the
existing client and play a full investigation. The server runs no rendering, no audio, no UI, no environmental grids.

This is the **minimum viable dedicated server**. It covers Phase 1 and Phase 2 of the implementation plan from Plan 26,
plus the critical parts of Phase 3 needed to avoid panics. The meta-server, room codes, and multi-room orchestration are
out of scope — those are Phase 2 of the overall roadmap.

### Definition of Done

- A `unhaunter-server` binary exists and compiles.
- It starts headless with `unhaunter-server --host 5000`.
- A regular client connects via `--join <ip>:5000`.
- The client enters the lobby. The first connected client is the room owner and can select a map and start a mission.
- The mission runs: ghost spawns, players investigate, interactions work, ghost hunts, health drains, mission ends.
- The server process stays alive after mission end. Players return to lobby and can start another mission.
- No panics from missing render resources.
- No window, no GPU, no audio device required.
- The existing peer-hosted model (`--host` / `--join` on the regular binary) remains fully functional.

---

## Current State

### What Already Works

- TCP + JSONL networking with N-client support (Plans 21–25)
- Host-authoritative ghost AI with zero environmental grid dependencies (verified in Plan 26)
- Lobby with map/difficulty selection, late join, heartbeat, disconnect resilience
- `CliOptions` with `NetMode::Host { port, bind_addresses }` — CLI parsing via clap
- `is_host()` / `is_client()` run conditions based on `NetMode`
- `PlayerInput` sent client→host every frame with movement, interaction, gear usage
- `SnapshotMsg` broadcast host→all with full game state

### What's Missing

1. **No `--dedicated` flag** — no way to signal headless mode
2. **No `is_headless()` run condition** — can't gate visual systems
3. **`PlayerInput` lacks `sanity` / `mean_sound`** — server can't read client-computed sanity
4. **Health regen is inside `lose_sanity`** — tangled with grid reads, needs extraction
5. **No dedicated server binary** — only the full client binary exists
6. **Map loading panics without render resources** — `LoadLevelSystemParam` requires `Assets<CustomMaterial1>` and
   `Assets<Mesh>`, which come from `Material2dPlugin` (loaded by `UnhaunterRenderPlugin`)
7. **No `RoomOwner` concept** — lobby leader is implicit (the host player)
8. **Server has no `MainPlayer`** — systems with `With<MainPlayer>` queries will silently skip, which is correct, but
   lobby map/difficulty selection currently requires local input
9. **`is_host` is ambiguous** — conflates "simulation authority" (the server process) with "team captain" (the player
   who picks the map). In a dedicated server, ALL players act like clients — `is_host` gating for player-specific logic
   is wrong. Needs renaming and role separation.
10. **No lobby network messages for map/difficulty** — `RequestSelectMap`, `RequestSelectDifficulty`,
    `RequestStartMission` do not exist in `NetworkMessage`. Lobby selection is purely local UI on the host.
11. **`handle_player_death` requires `Persistent<PlayerProfileData>`** — this is a per-player resource that won't exist
    on the dedicated server. System needs splitting.
12. **Server has no state machine auto-transitions** — `autostart_net_game` pre-fills lobby data but does NOT
    auto-transition from `MainMenu` → `Lobby`. Server would be stuck at `MainMenu`. Same issue post-mission: server
    would be stuck at `Summary`.
13. **Server claims `NetworkId(1)` but has no body** — creates a phantom "Player 1" in the lobby.
14. **Sanity round-trip overwrite** — `client_sync.rs` overwrites `PlayerSprite.sanity` from snapshot for ALL players
    including `MainPlayer`. If sanity is client-authoritative, the client's local computation gets overwritten every
    frame by the server's relay.

---

## Architecture & Design Decisions (Post-Review)

These decisions were finalized after review of the initial draft. They apply across all steps.

### D1. Naming & Roles: `is_authority()` + `RoomOwner`

The overloaded term "Host" conflates two concepts that diverge in dedicated server mode:

| Concept              | Old Name    | New Name                   | Meaning                                                                                    | True for                                                                  |
| -------------------- | ----------- | -------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------- |
| Simulation authority | `is_host()` | **`is_authority()`**       | This process runs the authoritative game simulation (ghost AI, health, state transitions). | `Offline`, `Host`, `Dedicated`                                            |
| Team captain         | (implicit)  | **`RoomOwner(NetworkId)`** | Which player controls lobby decisions (map, difficulty, start mission).                    | Peer-Host: Host player. Dedicated: first client. Transfers on disconnect. |

- `is_client()` stays as-is (means "I'm connecting to someone else's server").
- The existing `Authority::Host` / `Authority::Client` enum in `uninteraction-core` stays unchanged — it already
  represents the right concept (simulation authority vs read-only for interactions).
- Every usage of `is_host()` in the codebase (~16 call sites in source code) must be renamed to `is_authority()`.
  **Lobby UI checks** ("can I see the start button?") must become `RoomOwner` checks, not `is_authority()` checks.

### D2. Sanity Authority: Client-Authoritative (Thin Server)

All data must have **one and only one source of truth**. Round-trips of data and dual mastership are banned.

- **Sanity**: **Client-Authoritative**. The client computes sanity from local grids and sends it to the server via
  `PlayerInput`. The server trusts this value, writes it to `PlayerSprite.sanity`, and relays it to other clients via
  `PlayerState` snapshots.
- **Health**: **Server-Authoritative**. The server computes damage (ghost attacks) and regeneration.
- **Grids**: Server does **not** run Light, Thermal, or Sound grids.

**Data flow (no round-trip):**

```
Client A computes sanity from local grids
  → sends sanity in PlayerInput to server
    → server writes to PlayerSprite.sanity (for client A's entity)
      → server includes in SnapshotMsg
        → Client B receives, updates its copy of Client A's PlayerSprite
        → Client A receives, IGNORES its own sanity from snapshot (keeps local computation)
```

The client is the sole writer. The server and other clients are mirrors. The round-trip is broken by the client skipping
its own sanity in the snapshot.

**Behavioral change in peer-hosted mode:** Previously, the host computed sanity for ALL players (including remote) from
host's grids. After our changes, `lose_sanity` only computes for `MainPlayer`. Remote player sanity comes from
`PlayerInput`. So the snapshot sanity for remote players changes from "host-computed from host's grids" to
"client-reported from client's grids." This is practically invisible (co-op game, smoothed averages converge).

Server-authoritative sanity (server computing simplified lighting) is deferred to a future plan.

### D3. Server Identity: `NetworkId(0)`

- The dedicated server claims **`NetworkId(0)`**. It has no physical body (`MainPlayer`) in the world.
- `PlayerRegistry::next_id` starts at `1` when `dedicated` (so first client = `NetworkId(1)`).
- `session_roster_system` must **not** add `NetworkId(0)` to `lobby_data.players` — the server is not a player.
- In peer-hosted mode, the Host remains `NetworkId(1)` and the registry starts at `2`, unchanged.

### D4. Lobby Control Protocol

Since the dedicated server has no GUI, lobby actions must be network messages sent by the **Room Owner** client:

- `NetworkMessage::RequestSelectMap { map_filepath: String }`
- `NetworkMessage::RequestSelectDifficulty { difficulty_id: String }`
- `NetworkMessage::RequestStartMission`

The server validates sender == `RoomOwner`, updates `LobbyData`, and broadcasts it. The client UI reflects `LobbyData`
received from the server.

### D5. `handle_player_death` Split

Avoid spaghetti code with if/else role-based branching. Split into two systems where each is wholly gated for one role:

1. **`detect_and_apply_death`**: Runs on authority (`is_authority`). For ALL dead players: insert `PlayerSpectating`,
   despawn held gear, emit `PlayerDied { network_id }` event. Does NOT touch `PlayerProfileData`.
2. **`update_profile_death_stats`**: Runs on every instance (authority and client). Reads `PlayerDied` event. If event
   ID matches `MainPlayer`, updates `Persistent<PlayerProfileData>` statistics. On dedicated server, no `MainPlayer`
   exists, so the profile block never runs.

### D6. Server State Machine: Auto-Transitions

- **Boot:** `Loading` → `MainMenu` (existing `continue_to_state`). `autostart_net_game` fires on `OnEnter(MainMenu)`,
  pre-fills lobby, and if `dedicated`, immediately sets `next_state(AppState::Lobby)`. Server briefly touches `MainMenu`
  state — that's fine.
- **Post-mission:** Server stays in `Summary` for 2 frames (so `SnapshotMsg` with summary state goes out to clients),
  then sets `next_state(AppState::Lobby)`. Clients follow because `client_sync.rs` already transitions when
  `msg.app_state == AppState::Lobby`.

---

## Work Breakdown

### Step 1: Add `dedicated` flag to `CliOptions` + Rename `is_host` to `is_authority`

**Files:**

- `crates/uncommon-app-core/src/cli.rs`
- ~16 usage sites across `unplayer-plugin`, `unghost-plugin`, `untruck-plugin`, `uninteraction-plugin`, `unlobby-plugin`

**Changes (Part A — `dedicated` flag):**

- Add `pub dedicated: bool` field to `CliOptions` (default `false`).
- Add `pub fn is_headless(cli: Res<CliOptions>) -> bool` run condition that returns `cli.dedicated`.
- The existing `is_host()` function must return `true` when `dedicated` is true, since the dedicated server IS the host.
  Currently `is_host` returns `!matches!(cli.net_mode, NetMode::Join { .. })` — this already works for `NetMode::Host`.
  No change needed.

**Why `dedicated: bool` instead of a new `NetMode::Dedicated` variant:**

- `NetMode::Dedicated` would require touching every `match` on `NetMode` across the codebase (dozens of sites).
- A `bool` flag is additive — `dedicated: true` combined with `NetMode::Host { port, bind }` means "host, but headless."
  All existing `is_host()` / `is_client()` checks work unchanged.
- The flag can later become part of a `NetMode::Dedicated` variant if the design warrants it, but for now the simpler
  approach avoids churn.

**Validation:** `cargo clippy` — the field is just a `bool` on a `Resource`, no possible breakage.

**Changes (Part B — Rename `is_host` → `is_authority`):**

Rename the function `is_host` to `is_authority` in `cli.rs`. Logic stays the same:
`!matches!(cli.net_mode, NetMode::Join { .. })`. This is true for `Offline`, `Host`, and (by extension) `Dedicated`.

Rename all call sites. Known usage sites from grep:

| File                                                                  | Current Usage            | New Usage                |
| --------------------------------------------------------------------- | ------------------------ | ------------------------ |
| `crates/unplayer-plugin/src/systems/grabdrop.rs` L238                 | `.run_if(is_host)`       | `.run_if(is_authority)`  |
| `crates/unghost-plugin/src/systems/ghost_ai/mod.rs` L195-199          | 5× `.run_if(is_host)`    | `.run_if(is_authority)`  |
| `crates/unghost-plugin/src/systems/gis/selection.rs` L27              | `.run_if(is_host)`       | `.run_if(is_authority)`  |
| `crates/unghost-plugin/src/systems/gis/execution.rs` L168             | `.run_if(is_host)`       | `.run_if(is_authority)`  |
| `crates/unghost-plugin/src/systems/dynamic_behavior_update.rs` L169   | `.run_if(is_host)`       | `.run_if(is_authority)`  |
| `crates/untruck-plugin/src/truckgear.rs` L10                          | `.run_if(is_host)`       | `.run_if(is_authority)`  |
| `crates/uninteraction-plugin/src/systems/mod.rs` L60                  | `if is_host(cli)`        | `if is_authority(cli)`   |
| `crates/untruck-plugin/src/systems/in_truck_manager.rs` L16,40        | `is_host(cli)`           | `is_authority(cli)`      |
| `crates/unplayer-plugin/src/systems/movement.rs` L146                 | inline `is_host` check   | use `is_authority` logic |
| `crates/unplayer-plugin/src/systems/keyboard.rs` L17                  | inline `is_host` check   | use `is_authority` logic |
| `crates/unplayer-plugin/src/systems/waypoint.rs` L238                 | `is_host(cli)`           | `is_authority(cli)`      |
| `crates/unplayer-plugin/src/systems/input/mouse_interaction.rs` L25   | `is_host(cli)`           | `is_authority(cli)`      |
| `crates/unlobby-plugin/src/systems/lobby_main.rs` L62,110,195,330,336 | local `is_host` variable | See below                |

**Lobby UI special case:** In `lobby_main.rs`, the local `is_host` variable gates who sees map/difficulty selectors and
the "Start Mission" button. These should become **`RoomOwner` checks** (Step 10), not `is_authority()` checks. For now,
rename to `is_authority` for consistency, and add a `// TODO: Replace with RoomOwner check (Step 10)` comment. Step 10
will finalize this.

---

### Step 2: Add `sanity` and `mean_sound` to `PlayerInput` protocol

**Files:**

- `crates/unplayer-core/src/components.rs` — add fields to `PlayerInput` component
- `crates/unnet-core/src/messages.rs` — add fields to `NetworkMessage::PlayerInput` variant
- `crates/unnet-plugin/src/systems/client_input.rs` — send `sanity` and `mean_sound` from local player
- `crates/unnet-plugin/src/systems/host_input.rs` — receive and store on `PlayerInput` component

**Changes to `PlayerInput` component:**

```rust
pub struct PlayerInput {
    // ... existing fields ...

    /// Client-computed sanity value (0–100). Server uses this for ghost rage.
    pub sanity: f32,

    /// Client-computed mean sound level. Server uses this for ghost rage.
    pub mean_sound: f32,
}
```

Default both to `100.0` (sanity) and `0.0` (mean_sound) — safe initial values that produce minimal ghost rage.

**Changes to `NetworkMessage::PlayerInput`:**

Add `sanity: f32` and `mean_sound: f32` fields to the variant.

**Changes to `client_input.rs`:**

In `client_send_input_system`, read `PlayerSprite.sanity` and `PlayerSprite.mean_sound` from the local player entity and
include them in the `NetworkMessage::PlayerInput`. This requires adding `PlayerSprite` to the query.

**Changes to `host_input.rs`:**

In `host_apply_input_system`, copy `sanity` and `mean_sound` from the received message to the `PlayerInput` component.

**Protocol compatibility note:** This changes the JSON schema of `NetworkMessage::PlayerInput`. Old clients connecting
to a new server (or vice versa) will fail to deserialize the message. This is acceptable — multiplayer requires matching
versions. The `Hello` handshake already carries a `version` field for mismatch detection.

**Validation:** `cargo clippy`. Verify round-trip: add a temporary `info!()` in `host_apply_input_system` to log
received sanity values.

---

### Step 3: Propagate `PlayerInput` sanity/mean_sound to `PlayerSprite`

**Files:**

- `crates/unplayer-plugin/src/systems/sanityhealth.rs` — new system
- `crates/unplayer-plugin/src/systems/setup.rs` (or wherever `app_setup` is) — register the system

**New system: `server_apply_client_sanity`**

```rust
fn server_apply_client_sanity(
    mut q_player: Query<(&PlayerInput, &mut PlayerSprite), Without<MainPlayer>>,
) {
    for (input, mut sprite) in &mut q_player {
        sprite.sanity = input.sanity.clamp(0.0, 100.0);
        sprite.mean_sound = input.mean_sound.clamp(0.0, 100000.0);
    }
}
```

This runs on the authority only (`.run_if(is_authority)`), in `Update`, and only for non-`MainPlayer` entities (remote
players). For the local `MainPlayer` (in peer-hosted mode), `lose_sanity` already computes sanity from grids. On the
dedicated server there is no `MainPlayer`, so all players go through this path.

**Validation:** `cargo clippy`. Manual test: host with regular binary, connect client, verify client's sanity appears
correctly on host's player entity.

---

### Step 4: Extract health regen from `lose_sanity`

**Files:**

- `crates/unplayer-plugin/src/systems/sanityhealth.rs`

**Problem:** `lose_sanity` currently computes sanity (reads grids) AND applies health regen (no grids needed) in the
same function. On the dedicated server, `lose_sanity` won't run (it reads `LightGrid`, `ThermalGrid`, `SoundGrid` which
don't exist). But health regen must still happen.

**Changes:**

Extract the health regen block into a new system `health_regen`:

```rust
fn health_regen(
    time: Res<Time>,
    mut qp: Query<&mut PlayerSprite, (Without<InTruck>, Without<PlayerSpectating>)>,
    difficulty: Res<CurrentDifficulty>,
) {
    let dt = time.delta_secs();
    for mut ps in &mut qp {
        if ps.health < 100.0 && ps.health > 0.0 {
            ps.health += (0.1 * dt + (1.0 - ps.health / 100.0) * dt * 10.0)
                * difficulty.0.health_recovery_rate;
        }
        if ps.health > 100.0 {
            ps.health = 100.0;
        }
    }
}
```

Remove the health lines from `lose_sanity`. Register `health_regen` alongside `lose_sanity` in the system schedule. Both
run on authority (peer-hosted or dedicated). On the dedicated server, only `health_regen` runs (sanity comes from
`PlayerInput`).

Additionally, `recover_sanity` (in-truck recovery) also does health regen for players in the truck. That health regen
can stay in `recover_sanity` since truck recovery is simple and the dedicated server will still run it. Alternatively,
make `health_regen` work for both in-truck and out-of-truck players by removing the `Without<InTruck>` filter and
adjusting the rate. But that's a behavioral change — keep them separate for now.

**Registration:** `health_regen` runs on all instances (host and client). The client already overwrites health from the
snapshot, so the local regen is overwritten immediately. No harm.

Alternatively, gate `health_regen` with `.run_if(is_authority)` so it only runs on the authoritative side. Cleaner.

**Validation:** `cargo clippy`. Play a peer-hosted game, verify health still regens after ghost damage.

---

### Step 5: Gate `lose_sanity` behind `is_headless().not()`

**Files:**

- `crates/unplayer-plugin/src/systems/setup.rs` (or equivalent `app_setup`)

**Changes:**

`lose_sanity` reads `LightGrid`, `ThermalGrid`, `SoundGrid` — resources that won't exist on the dedicated server. It
must NOT run on the server.

Add `.run_if(not(is_headless))` to `lose_sanity`. On the dedicated server, sanity comes from `PlayerInput` (Step 3).

`recover_sanity` (in-truck recovery) also reads no grids — it's pure arithmetic on `crazyness`/`sanity`/`health`. It's
safe to run on the server. In fact, it should run on the server so that in-truck health recovery is authoritative.

**But wait:** On the dedicated server, `recover_sanity` will modify `sanity` for in-truck players. In the thin server
model, sanity is client-reported. If the server also modifies sanity, the snapshot will carry the server's value, and
the client will see a jump. **Resolution:** On the dedicated server, `recover_sanity` should only do health regen, not
sanity recovery. OR: leave `recover_sanity` running everywhere (it already does today), and accept that the in-truck
sanity recovery is slightly different between client and server — it converges quickly and is not observable by other
players (in-truck players' sanity isn't tactically relevant).

**Recommended approach:** Leave `recover_sanity` unchanged for now. The client's local sanity value is what the client
reports in `PlayerInput`; the server's `recover_sanity` modifies `PlayerSprite.sanity`, but `server_apply_client_sanity`
(Step 3) overwrites it with the client-reported value every frame. The client-reported value wins. This means in-truck
sanity recovery is effectively client-driven, which is fine.

**Additionally: Add `With<MainPlayer>` to `lose_sanity` and `recover_sanity` queries.** This ensures sanity simulation
only runs for the local player. On the dedicated server (no `MainPlayer`), these systems do nothing. On a peer host,
they simulate the host player only; remote players get sanity from `server_apply_client_sanity`. This also fixes an
existing bug where `lose_sanity` computes for remote players using the host's grids (which are at the host's camera
position, not the remote player's position) — a race condition that was practically invisible but architecturally wrong.

**Sanity snapshot overwrite fix:** In `client_sync.rs` (lines 869-870), the snapshot handler overwrites
`player_sprite.sanity` for ALL players, including `MainPlayer`. With client-authoritative sanity, the client must skip
its own sanity from the snapshot. Add a guard: if the entity has `MainPlayer`, skip the `sanity` field overwrite.
`health` is still overwritten from snapshot (server-authoritative). This change is part of this step.

**Validation:** `cargo clippy`. Verify the sanity system registration order: `server_apply_client_sanity` should run
AFTER `recover_sanity` so the client-reported value takes precedence for remote players.

---

### Step 6: Register placeholder render resources for headless mode

**Files:**

- `crates/unrender-plugin/src/plugin.rs` (or a new module within it)
- Possibly `crates/unrender-core` if types are needed

**Problem:** `LoadLevelSystemParam` (in `unmapload-plugin`) requires `ResMut<Assets<CustomMaterial1>>` and
`ResMut<Assets<Mesh>>`. These are normally initialized by `Material2dPlugin<CustomMaterial1>` (from
`UnhaunterRenderPlugin`) and Bevy's `MeshPlugin` (from `DefaultPlugins`). On a headless server, neither runs.

Without these resources, the ECS system parameter resolution panics before the system even executes.

**Approach: Stub resources.**

Register the missing `Assets<T>` resources manually in the dedicated server's app setup:

```rust
// In the dedicated server binary or a headless setup function:
app.init_resource::<Assets<CustomMaterial1>>();
app.init_resource::<Assets<Mesh>>();
// Also init Assets<Image> if needed by asset loader collections
app.init_resource::<Assets<Image>>();
```

`Assets<T>` is a Bevy-provided generic resource. Calling `init_resource` creates an empty asset store. Handles created
into it will be valid (non-panicking) but the assets won't be rendered.

**Problem within `process_and_spawn_tile`:** This function calls `materials.get(handle)` and may panic or produce
garbage if the source material handle doesn't resolve. Need to verify what happens. If it panics, the tile spawning
function needs a headless branch that skips visual component creation.

**Investigation needed:** Read `process_and_spawn_tile` to determine if a `.get()` on a missing material handle panics
or returns `None`. If `None`, the function must handle it gracefully. If it panics, we need an `if is_headless` guard.

Since `is_headless` is a `Res<CliOptions>` check, and `process_and_spawn_tile` is a regular function (not a system), it
would need the `CliOptions` passed down or checked at the call site.

**Alternative approach:** Make `LoadLevelSystemParam` include an `Option<ResMut<Assets<CustomMaterial1>>>` instead of
`ResMut<Assets<CustomMaterial1>>`. Then check `.is_some()` before using it. This is a cleaner long-term fix but touches
the type signature.

**Recommended approach for this step:** Register stub `Assets<T>` resources. In `process_and_spawn_tile`, replace any
`.unwrap()` or direct `[]` access on material `.get()` with graceful fallbacks (skip visual components if material is
missing). This keeps the function working in both modes without an explicit `is_headless` parameter.

**Validation:** Start the dedicated server, load a map, verify no panic. The map entities should still have `Position`,
`BoardPosition`, `Behavior`, etc. — just no visual components.

---

### Step 7: Create the dedicated server binary

**Files:**

- `unhaunter/src/bin/dedicated.rs` (new file)
- `unhaunter/Cargo.toml` (add `[[bin]]` entry)

**The binary:**

```rust
use clap::Parser;
use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use std::time::Duration;

#[derive(Parser, Debug)]
#[clap(name = "unhaunter-server")]
struct Args {
    #[clap(long, default_value = "5000")]
    host: u16,

    #[clap(long)]
    bind: Vec<String>,

    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let args = Args::parse();

    let mut bind_addresses = args.bind.clone();
    if bind_addresses.is_empty() {
        bind_addresses.push("::".to_string());
        bind_addresses.push("0.0.0.0".to_string());
    }

    let cli_options = uncommon-app_core::cli::CliOptions {
        dedicated: true,
        net_mode: uncommon-app_core::cli::NetMode::Host {
            port: args.host,
            bind_addresses,
        },
        verbose: args.verbose,
        mute: true,
        ..Default::default()
    };

    // ... build headless App (see below)
}
```

**App setup:**

Instead of `DefaultPlugins`, use `MinimalPlugins` + selective additions.

```rust
fn build_headless_app(cli_options: CliOptions) {
    let mut app = App::new();

    app.insert_resource(cli_options);

    app.add_plugins(MinimalPlugins.set(
        ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0))
    ));

    // Asset loading (file I/O, no GPU)
    app.add_plugins(AssetPlugin::default());

    // Transform/Hierarchy (used by game logic)
    app.add_plugins(TransformPlugin);
    app.add_plugins(HierarchyPlugin);

    // State machine
    app.add_plugins(StatesPlugin);

    // Stub render resources
    app.init_resource::<Assets<Image>>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<unrender_core::...::CustomMaterial1>>();

    // --- Game plugins (simulation + network only) ---
    app.add_plugins((
        UnhaunterEnginePlugin,
        UnhaunterDifficultyPlugin,
        UnhaunterNetPlugin,
        UnhaunterTmxMapPlugin,
        UnhaunterMapLoadPlugin,
        UnhaunterRenderPlugin,  // needs is_headless gating (Step 8)
        UnhaunterGhostPlugin,
        UnhaunterInteractionPlugin,
        UnhaunterMissionPlugin,
        UnhaunterLobbyPlugin,
        UnhaunterPlayerPlugin,  // needs is_headless gating (Step 8)
        UnhaunterGearPlugin,
        ClassicModePlugin,      // needs is_headless gating (Step 8)
        UnhaunterTruckPlugin,   // needs is_headless gating (Step 8)
    ));

    app.run();
}
```

**Key question: which plugins can we load vs which will panic?**

Several plugins in the KEEP list register systems that query render resources (cameras, materials, sprites). The systems
themselves may be fine (they query optional components, or the entities simply don't exist), but the **plugin
registration** might try to add Bevy plugins that require render backends (e.g., `Material2dPlugin`).

This is why Step 8 exists — each plugin's `impl Plugin` needs to be audited for unconditional render-backend
registrations.

**Validation:** The binary should compile. It may not run yet without Step 8 fixes.

---

### Step 8: Audit and gate plugin registrations for headless mode

**Files:** Multiple plugin.rs files across crates.

Each plugin listed as KEEP or KEEP PARTIALLY in Plan 26 must be checked for:

1. **Bevy render plugin registrations** (`Material2dPlugin`, `UiMaterialPlugin`, `SpritePlugin`, etc.) — these panic
   without a render backend.
2. **System registrations that query render-only resources** — these won't panic (the query just returns no results),
   but waste cycles.
3. **Resource initialization that depends on render plugins** — these panic if the underlying `Assets<T>` doesn't exist.

**Plugin-by-plugin audit:**

#### `UnhaunterRenderPlugin` (`crates/unrender-plugin/src/plugin.rs`)

This is the most critical. It registers `Material2dPlugin<CustomMaterial1>`, `UiMaterialPlugin<UIPanelMaterial>`, and
likely other render-dependent plugins. The dedicated server needs:

- `BoardTopology`, `BoardCollisionField`, `SpriteDB`, `RoomDB` — simulation resources
- `board_sync` systems — map entity hydration
- Entity hydration (assigns `Behavior`, `Position`, etc.)

It does NOT need:

- `Material2dPlugin`, `UiMaterialPlugin`
- `apply_perspective`, `set_window_icon`
- Sprite/visual sync systems

**Approach:** In `plugin.rs`, check `cli.dedicated` (or add an `is_headless` method on `CliOptions`). Skip
`Material2dPlugin` and visual system registration when headless. The plugin needs access to `CliOptions` during
registration — since `Plugin::build(&self, app: &mut App)` has access to `app.world()`, it can read the
`Res<CliOptions>` from the world (it was inserted before plugins are added).

Alternatively, the plugin can accept a configuration flag: `UnhaunterRenderPlugin { headless: bool }`. This avoids
reading from the world during plugin initialization.

**Wait — can `Plugin::build` read resources?** In Bevy, resources inserted via `app.insert_resource()` before
`app.add_plugins()` ARE available in `app.world()` during `Plugin::build`. So `app.world().get_resource::<CliOptions>()`
works. However, this is fragile and depends on initialization order.

**Better approach:** Pass the `dedicated` flag as plugin configuration.

```rust
pub struct UnhaunterRenderPlugin {
    pub headless: bool,
}

impl Plugin for UnhaunterRenderPlugin {
    fn build(&self, app: &mut App) {
        if !self.headless {
            app.add_plugins(Material2dPlugin::<CustomMaterial1>::default());
            app.add_plugins(UiMaterialPlugin::<UIPanelMaterial>::default());
            // ... visual systems
        }
        // Always register simulation resources + systems
        systems::app_setup(app, self.headless);
    }
}
```

**Problem with this approach:** The convention says plugins export only the plugin itself. Changing the API from
`UnhaunterRenderPlugin` (unit struct) to `UnhaunterRenderPlugin { headless: bool }` changes all call sites (both
`app.rs` and `dedicated.rs`). But the regular client always passes `headless: false`, so the change is minimal.

**Simpler alternative:** Read `CliOptions` from the world during `build`. Since `CliOptions` is inserted before any
plugins are added (see `app.rs` line 47: `app.insert_resource(cli_options.clone())`), this is safe.

```rust
impl Plugin for UnhaunterRenderPlugin {
    fn build(&self, app: &mut App) {
        let headless = app.world()
            .get_resource::<CliOptions>()
            .map_or(false, |cli| cli.dedicated);

        if !headless {
            app.add_plugins(Material2dPlugin::<CustomMaterial1>::default());
            // ...
        }
        // Always: simulation resources
    }
}
```

This requires no API change. All existing call sites remain `UnhaunterRenderPlugin`. The dedicated server just sets
`dedicated: true` in `CliOptions` before adding plugins.

**Recommendation:** Use the `CliOptions` world read approach. It preserves the plugin API convention and requires zero
changes to the regular client's `app.rs`.

#### `UnhaunterPlayerPlugin` (`crates/unplayer-plugin/src/plugin.rs`)

Needs:

- Movement system, position tracking, interaction dispatch, grabdrop

Does NOT need:

- Input systems (keyboard, mouse), styling, camera, sanity (gated in Step 5), viewer sync, walk target indicators

**Approach:** Same pattern — read `CliOptions.dedicated` in `build()` or `app_setup()`, skip visual/input systems.

#### `UnhaunterGhostPlugin` (`crates/unghost-plugin/src/plugin.rs`)

Needs:

- Ghost AI (movement, enrage, warning, fade-out, scale glitch)
- Behavior dynamics, interaction selection/execution
- `sync_ghost_emitters` (configures emitter components from clarity values — clients read these from snapshot data to
  drive their local grids). **CHECK:** Does the dedicated server need `sync_ghost_emitters`? The server doesn't run
  environmental grids, so emitters have no consumers. But the snapshot might read emitter component values. If so, the
  system is needed. If the snapshot only reads `GhostBehaviorDynamics` clarity values directly, emitters are
  unnecessary.

Does NOT need:

- `ghost_visual_sync`, `ghost_influence_visual_sync`, `spawn_ghost_orb_particles`

#### `ClassicModePlugin` (`crates/unclassic-mode-plugin/src/plugin.rs`)

Needs:

- `classic_mode_orchestrator` (spawns ghost, breach, players)
- `spawn_joined_player`

Does NOT need:

- Evidence perception, game UI, hints, looking gear

#### `UnhaunterTruckPlugin` (`crates/untruck-plugin/src/plugin.rs`)

Needs:

- Truck gear initialization, loadout management, journal state

Does NOT need:

- Truck UI, truck camera

#### `UnhaunterLobbyPlugin` (`crates/unlobby-plugin/src/plugin.rs`)

Needs:

- Lobby state management, player roster, map/difficulty selection handling

Does NOT need:

- Lobby UI rendering

#### `UnhaunterGearPlugin` (`crates/ungear-plugin/src/plugin.rs`)

Needs:

- `GearSpawnerRegistry`, gear entity management

Does NOT need:

- Gear rendering, sound events (the `SoundEmitter` SystemParam used in ghost AI wraps `AssetServer` + `Time` +
  `MessageWriter<SoundEvent>`, all of which exist headless — the events fire but nothing plays them)

**Validation:** Start dedicated server binary. If it panics during plugin registration, the stack trace shows which
plugin and which Bevy sub-plugin caused the panic. Fix that plugin's registration. Iterate until startup succeeds.

---

### Step 9: Handle `MainPlayer` absence on the dedicated server

**Files:**

- `crates/unclassic-mode-plugin/src/systems/orchestrator.rs`
- `crates/unnet-plugin/src/systems/connection.rs`
- `crates/unnet-plugin/src/resources.rs`

**Analysis:** The dedicated server has no `MainPlayer` entity. Systems with `With<MainPlayer>` queries will return empty
results and do nothing. This is the desired behavior for:

- Input systems (no local player to control)
- Camera systems (no camera to follow)
- Sanity/health visual effects (no screen to show overlays)

Systems with `Without<MainPlayer>` queries (like `host_apply_input_system`'s player query) will see ALL player entities.
This is correct — on the dedicated server, every player is remote.

**Concrete changes to `classic_mode_orchestrator`:**

`classic_mode_orchestrator` (lines 131–135) determines which players to spawn locally:

```rust
let player_ids_to_spawn: Vec<usize> = match p.cli.net_mode {
    uncommon-app_core::cli::NetMode::Offline => vec![1],
    uncommon-app_core::cli::NetMode::Host { .. } => vec![1],
    uncommon-app_core::cli::NetMode::Join { .. } => vec![],
};
```

On a dedicated server (`NetMode::Host` + `dedicated: true`), this incorrectly spawns Player 1 as `MainPlayer`. Change:

```rust
uncommon-app_core::cli::NetMode::Host { .. } => {
    if p.cli.dedicated {
        vec![] // Dedicated server spawns NO local players
    } else {
        vec![1] // Peer host spawns Player 1
    }
},
```

**Concrete changes for `NetworkId(0)` (Decision D3):**

1. In `startup_network_system` (`connection.rs` L308): when `dedicated`, set `local_id.0 = Some(NetworkId(0))` instead
   of `NetworkId(1)`.
2. In `PlayerRegistry` (`resources.rs` L107): when `dedicated`, set `next_id: 1` (so first client = `NetworkId(1)`). The
   current default is `next_id: 2` (reserves 1 for host).
3. In `session_roster_system` (`connection.rs` L63-76): skip adding `NetworkId(0)` to `lobby_data.players` — the server
   is not a player and should not appear in the lobby roster.

**Remote Players:** `spawn_joined_player` (line 479) handles spawning players connected via network. It already runs
only for `NetMode::Host`. It correctly spawns `PlayerSprite` without `MainPlayer`, `Viewer`, `SpatialListener`, etc.
With `NetworkId(0)` for the server, the first connecting client gets `NetworkId(1)` and appears as Player 1.

---

### Step 10: Room owner tracking

**Files:**

- `crates/unnet-core/src/resources.rs` (or new file) — `RoomOwner` resource
- `crates/unnet-plugin/src/systems/setup.rs` — new system for room owner management
- `crates/unnet-plugin/src/systems/host_input.rs` — gate lobby actions by room owner

**New resource:**

```rust
/// The NetworkId of the player who has lobby leader privileges.
/// Only meaningful on the host/dedicated server.
#[derive(Resource, Default)]
pub struct RoomOwner(pub Option<NetworkId>);
```

**Behavior:**

- When the first client connects, they become the room owner.
- When the room owner disconnects, ownership transfers to the next connected player (by join order).
- Lobby actions (select map, select difficulty, start mission) are checked against the room owner's `NetworkId`. If the
  sender is not the room owner, the action is ignored (with a warning log).

**In peer-hosted mode:** The host player is implicitly the room owner. `RoomOwner` can be initialized to the host's
`NetworkId` for consistency, but existing code doesn't check it — so this is a no-op for the current model.

**Note:** The room owner is NOT the simulation authority. The simulation authority is always the host process (whether
peer-hosted or dedicated). The room owner only controls lobby UI actions.

**This step is required.** Without `RoomOwner`, there is no way for the dedicated server to know which client is allowed
to select maps and start missions. In peer-hosted mode, `RoomOwner` is initialized to `NetworkId(1)` (the host's own
player).

---

### Step 11: PerlinNoise optimization for server

**Files:**

- `crates/unnoise-core/src/perlin.rs`

**Problem:** `PerlinNoise::new(seed)` allocates a 4000×4000 `Vec<Vec<f32>>` = 64 MB. This is the dominant fixed cost per
server process.

**Solution:** Add a `PerlinNoise::lightweight(seed)` constructor that stores only the seed and uses the `noise::Perlin`
crate to compute values on-the-fly. The existing `get()` method becomes:

```rust
pub fn get(&self, x: f32, y: f32) -> f32 {
    match &self.table {
        Some(table) => { /* existing table lookup */ },
        None => { /* compute on the fly using noise::Perlin */ },
    }
}
```

Change the field from `Vec<Vec<f32>>` to `Option<Vec<Vec<f32>>>`. When `None`, compute on-the-fly.

In the dedicated server binary, initialize `PerlinNoise` with `lightweight()` instead of `new()`. The regular client
keeps using `new()` for the precomputed table.

**Alternative:** Simply use a smaller table (1000×1000 = 4 MB). Less code change, but still allocates memory. The
lightweight approach is cleaner for the server's ~10 lookups/frame.

**This step is optional for the MVP** but strongly recommended — it reduces per-process memory from ~112 MB to ~48 MB,
allowing ~20 concurrent instances on a 1 GB VPS instead of ~8.

---

### Step 12: Split `handle_player_death` into two systems (Decision D5)

**Files:**

- `crates/unplayer-plugin/src/systems/sanityhealth.rs`

**Current state:** `handle_player_death` (line 191) does two things in one system:

1. For ALL dead players: insert `PlayerSpectating`, despawn held gear.
2. For `MainPlayer` only: update `Persistent<PlayerProfileData>` statistics (death count, insurance).

The system requires `ResMut<Persistent<PlayerProfileData>>` as a parameter. On the dedicated server, this resource does
not exist (it's per-player, stored on each player's machine). The system would panic at parameter resolution.

**Changes — Create two systems:**

**System 1: `detect_and_apply_death`**

- Gate: `.run_if(is_authority)` — only the server decides when a player dies.
- Query: all players with `health <= 0` who don't already have `PlayerSpectating`.
- Actions: insert `PlayerSpectating`, despawn held gear entities, emit `PlayerDied { network_id: NetworkId }` event.
- Does NOT access `PlayerProfileData`.

```rust
fn detect_and_apply_death(
    mut commands: Commands,
    q_dead: Query<(Entity, &NetworkId, &PlayerSprite, &GearSlotsData), (Without<PlayerSpectating>,)>,
    mut ev_death: MessageWriter<PlayerDied>,
) {
    for (entity, net_id, sprite, gear_slots) in &q_dead {
        if sprite.health <= 0.0 {
            commands.entity(entity).insert(PlayerSpectating);
            // Despawn gear...
            ev_death.write(PlayerDied { network_id: *net_id });
        }
    }
}
```

**System 2: `update_profile_death_stats`**

- Gate: runs on all instances (authority and client).
- Reads `PlayerDied` events. If event `network_id` matches the local `MainPlayer`'s `NetworkId`, update
  `Persistent<PlayerProfileData>`.
- On dedicated server: no `MainPlayer` exists → event is read but nothing matches → no profile access needed.
- On client: `MainPlayer` exists → if it's the one that died, update profile stats.
- `PlayerProfileData` is accessed via `Option<ResMut<Persistent<PlayerProfileData>>>` to be safe.

```rust
fn update_profile_death_stats(
    mut ev_death: MessageReader<PlayerDied>,
    q_main: Query<&NetworkId, With<MainPlayer>>,
    mut profile: Option<ResMut<Persistent<PlayerProfileData>>>,
) {
    for ev in ev_death.read() {
        if let Ok(main_net_id) = q_main.single() {
            if *main_net_id == ev.network_id {
                if let Some(ref mut profile) = profile {
                    // Update death stats...
                }
            }
        }
    }
}
```

**New event type:** `PlayerDied { network_id: NetworkId }` — register with `app.add_message::<PlayerDied>()`.

**Validation:** `cargo clippy`. Play peer-hosted game, verify death still triggers spectate mode and profile stats
update.

---

### Step 13: Lobby Control Protocol (Decision D4)

**Files:**

- `crates/unnet-core/src/messages.rs` — new `NetworkMessage` variants
- `crates/unlobby-plugin/src/systems/lobby_main.rs` — client-side: send requests instead of writing resources directly
- `crates/unlobby-plugin/src/systems/map_select.rs` — client-side: send `RequestSelectMap` on click
- `crates/unlobby-plugin/src/systems/difficulty_select.rs` — client-side: send `RequestSelectDifficulty` on click
- `crates/unnet-plugin/src/systems/host_input.rs` (or new file) — server-side: handle requests

**New message variants:**

```rust
// In NetworkMessage enum:
RequestSelectMap { map_filepath: String },
RequestSelectDifficulty { difficulty_id: String },
RequestStartMission,
```

**Client-side changes:**

Currently, map/difficulty selection writes directly to local `LobbyData`. This works in peer-hosted mode because the
host IS the authority. For dedicated servers:

1. **Map select (`map_select.rs`):** When the user clicks a map, instead of (or in addition to) writing
   `lobby_data.selected_map`, send `RequestSelectMap { map_filepath }` to the server.
2. **Difficulty select (`difficulty_select.rs`):** Same pattern — send `RequestSelectDifficulty { difficulty_id }`.
3. **Start mission (`lobby_main.rs`):** Instead of directly transitioning `AppState`, send `RequestStartMission`.

**For backward compatibility with peer-hosted mode:** The peer host can either:

- (a) Also send requests to itself (unified flow), OR
- (b) Keep the direct-write path for peer-hosted and only use messages for dedicated.

Option (a) is cleaner long-term. The host process would handle its own messages the same way it handles client messages.
But this is more refactoring. For MVP, option (b) is acceptable — gate the send-request path with
`if cli.dedicated || is_client` and keep the direct path for peer host.

**Server-side handler:**

New system `handle_lobby_requests` (runs on authority, in `Update`, gated to `AppState::Lobby`):

```rust
fn handle_lobby_requests(
    mut ev_reader: MessageReader<NetworkDataEvent>,
    room_owner: Res<RoomOwner>,
    mut lobby_data: ResMut<LobbyData>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    for ev in ev_reader.read() {
        let Some(sender) = ev.source else { continue };
        if room_owner.0 != Some(sender) {
            warn!("Lobby request from non-owner {:?}, ignoring", sender);
            continue;
        }
        match &ev.message {
            NetworkMessage::RequestSelectMap { map_filepath } => {
                lobby_data.selected_map = Some(map_filepath.clone());
            }
            NetworkMessage::RequestSelectDifficulty { difficulty_id } => {
                lobby_data.selected_difficulty = Some(difficulty_id.clone());
            }
            NetworkMessage::RequestStartMission => {
                if lobby_data.selected_map.is_some() {
                    next_app_state.set(AppState::InGame);
                }
            }
            _ => {}
        }
    }
}
```

The server already broadcasts `LobbyData` changes to all clients via the existing `lobby_broadcast_system`. Clients
receive `LobbyState` messages and update their local `LobbyData`.

**Validation:** `cargo clippy`. Connect client to dedicated server. Client selects map → server updates LobbyData → all
clients see the selection.

---

### Step 14: Server State Machine Auto-Transitions (Decision D6)

**Files:**

- `crates/unnet-plugin/src/systems/connection.rs` — amend `autostart_net_game`
- `crates/unsummary-plugin` or `crates/unmission-plugin` — post-mission transition

**Boot sequence (MainMenu → Lobby):**

`autostart_net_game` already fires on `OnEnter(AppState::MainMenu)` and pre-fills lobby data from CLI args. Add at the
end of this function:

```rust
if cli.dedicated {
    next_app_state.set(AppState::Lobby);
    info!("Dedicated server: auto-transitioning to Lobby");
}
```

The server briefly touches `MainMenu` state. All `OnEnter(MainMenu)` / `OnExit(MainMenu)` systems fire, but since
there's no UI, no window, no audio, they should be harmless. Step 8 (plugin audit) must verify this — if any
`OnEnter(MainMenu)` system panics on headless, it must be gated with `not(is_headless)`.

**Post-mission (Summary → Lobby):**

After the mission ends, the server enters `AppState::Summary`. It needs to return to `Lobby` automatically. Add a system
that runs on `OnEnter(AppState::Summary)` with `run_if(is_headless)`:

```rust
fn dedicated_summary_auto_return(
    mut frame_count: Local<u32>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    *frame_count += 1;
    if *frame_count >= 2 {
        next_app_state.set(AppState::Lobby);
        *frame_count = 0;
    }
}
```

Actually, an `OnEnter` system only runs once. For a 2-frame delay, register this as a regular `Update` system gated by
`in_state(AppState::Summary).and(is_headless)`, using a `Local<u32>` frame counter.

Clients follow because `client_sync.rs` already has:

```rust
if msg.app_state == AppState::Lobby && *current_state.get() != AppState::Lobby {
    next_app_state.set(AppState::Lobby);
}
```

**TODO/FIXME notes:** Leave comments at both transition points:

```rust
// FIXME(dedicated-server): This auto-transition is fragile. If any OnEnter/OnExit system
// for MainMenu or Summary panics on headless, it must be gated. See Plan 27 Step 8.
```

**Validation:** Start dedicated server. Verify it reaches `Lobby` state within 1 second of boot. After a mission, verify
it returns to `Lobby` within a few frames.

---

## Dependency Graph

```
Step 1: CliOptions.dedicated + is_headless() + rename is_host → is_authority
  │
  ├──→ Step 2: PlayerInput protocol change (sanity, mean_sound)
  │      │
  │      └──→ Step 3: server_apply_client_sanity system
  │
  ├──→ Step 4: Extract health_regen from lose_sanity
  │      │
  │      └──→ Step 5: Gate lose_sanity behind With<MainPlayer> + fix sanity snapshot overwrite
  │
  ├──→ Step 12: Split handle_player_death into detect_and_apply_death + update_profile_death_stats
  │
  ├──→ Step 6: Register stub render resources
  │
  ├──→ Step 8: Audit plugin registrations
  │      │
  │      └──→ Step 7: Create dedicated binary (needs Step 6, Step 8 done)
  │             │
  │             └──→ Step 9: MainPlayer absence + NetworkId(0) for server
  │
  ├──→ Step 10: RoomOwner resource + lobby UI updates (required)
  │      │
  │      └──→ Step 13: Lobby Control Protocol (RequestSelectMap, etc.)
  │
  ├──→ Step 14: Server state machine auto-transitions (independent)
  │
  └──→ Step 11: PerlinNoise optimization (optional, independent)
```

**Critical path:** Steps 1 → 10 → 13 → … → 6 → 8 → 7 → 9 → 14

Steps 2-5, 12, 11 can be done in parallel with the critical path after Step 1.

---

## Implementation Order

The dependency graph suggests this sequence, organized for incremental testability:

### Batch A: Refactoring & Authority Model (safe, no protocol changes, testable with peer-hosted mode)

1. **Step 1** — `CliOptions.dedicated` + `is_headless()` + rename `is_host` → `is_authority`. Trivial, unlocks
   everything.
2. **Step 12** — Split `handle_player_death` into `detect_and_apply_death` + `update_profile_death_stats`.
3. **Step 4** — Extract `health_regen` from `lose_sanity`.
4. **Step 5** — Add `With<MainPlayer>` to `lose_sanity`/`recover_sanity` + fix sanity snapshot overwrite in
   `client_sync.rs`.
5. **Step 10** — `RoomOwner` resource. Update lobby UI to use `RoomOwner` checks instead of `is_authority` for button
   visibility.

**Test checkpoint:** Run peer-hosted game. Everything works as before. Death, health regen, sanity, lobby all
functional. The `RoomOwner` is initialized to the host's `NetworkId(1)` in peer-hosted mode — no behavioral change.

### Batch B: Protocol Updates (requires client/server update together)

6. **Step 2** — Add `sanity`/`mean_sound` to `PlayerInput` + `NetworkMessage`. Protocol change.
7. **Step 3** — `server_apply_client_sanity` system.
8. **Step 13** — Lobby Control Protocol (`RequestSelectMap`, `RequestSelectDifficulty`, `RequestStartMission`).

**Test checkpoint:** Run peer-hosted game. Sanity is relayed correctly. Lobby actions work through new messages (or
still directly for peer-host if option (b)). No behavioral change visible to players.

### Batch C: Headless binary (requires Batch A + B)

9. **Step 6** — Register stub `Assets<T>` resources.
10. **Step 8** — Audit and gate plugin registrations.
11. **Step 7** — Create `unhaunter-server` binary.
12. **Step 9** — `MainPlayer` absence handling + `NetworkId(0)` for server.
13. **Step 14** — Server state machine auto-transitions (MainMenu → Lobby, Summary → Lobby).

**Test checkpoint:** Start `unhaunter-server --host 5000`. Connect with regular client. Verify: lobby works, Room Owner
can select map, mission loads, ghost spawns, interactions work, ghost hunts, health drains, mission ends, server returns
to lobby.

### Batch D: Optional Polish

14. **Step 11** — PerlinNoise lightweight mode.

**Test checkpoint:** Check server memory usage with lightweight Perlin.

---

## Risk Mitigation During Implementation

| Risk                                                | Detection                                         | Mitigation                                                                |
| --------------------------------------------------- | ------------------------------------------------- | ------------------------------------------------------------------------- |
| Plugin panics during headless startup               | Stack trace on `unhaunter-server` launch          | Fix one plugin at a time in Step 8; iterate                               |
| `process_and_spawn_tile` panics on missing material | Stack trace during map load                       | Add `.get()` → `Option` fallback or skip visual bundle                    |
| `bevy_asset_loader` fails without image assets      | Panic during `LoadingState` transition            | Register empty `Assets<Image>`, or stub asset collections                 |
| Client sanity value is stale / wrong scale          | Ghost rage is too low/high on dedicated server    | Log comparison: client-reported vs locally-computed (in peer-hosted mode) |
| `classic_mode_orchestrator` expects a local player  | Panic or missing ghost/breach on dedicated server | Add `if dedicated { skip MainPlayer spawn }` check                        |
| `SoundEmitter` fires events that nothing handles    | Silent — events accumulate in queue               | Acceptable; events are consumed each frame even if no audio plays         |
| `OnEnter(MainMenu)` systems panic on headless       | Stack trace during boot auto-transition           | Gate offending systems with `not(is_headless)` in Step 8                  |
| `is_authority` rename misses a call site            | `cargo clippy` compile error                      | Grep for `is_host` after rename; the old function name won't compile      |
| `RoomOwner` not set on peer-hosted mode             | Lobby buttons invisible for host                  | Initialize `RoomOwner` to `NetworkId(1)` in `startup_network_system`      |
| Lobby request messages not handled by peer-host     | Map selection broken after Step 13                | Keep direct-write path for peer-host (option b) or handle own messages    |

---

## What This Plan Does NOT Cover

These are deferred to future plans:

- **Meta-server** (HTTP REST room registry, process spawner) — Plan 28
- **Room codes and room browser** — Plan 28
- **`NetMode::Dedicated` variant** — unnecessary until meta-server needs it
- **Multi-room single process** — deferred until demand requires >20 concurrent rooms
- **Client-side prediction improvements** — orthogonal to dedicated server
- **Snapshot rate optimization** (60Hz → 20Hz) — performance tuning, post-MVP
- **Ghost visibility bias on server** (adding `Viewer` to all players) — minor quality improvement, post-MVP
- **Authentication, rate limiting, ban lists** — meta-server concerns
- **Server-authoritative sanity** — would require server to compute simplified lighting grids; deferred in favor of thin
  server model where clients are trusted for sanity
