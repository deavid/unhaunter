# CUJ Inspection Report — Multiplayer Hub

This document summarizes a technical inspection of the Multiplayer Hub implementation against the
[Critical User Journeys (CUJ)](05_critical_user_journeys.md).

## 1. Summary of Gaps & Successes

| Feature / Requirement     | Status          | Location / Technical Note                                                                                                                                                   |
| :------------------------ | :-------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Safe-Vocal Alphabet**   | ✅ **SUCCESS**  | Correctly implemented via hardcoded key mapping in [ui.rs](crates/unhub-plugin/src/ui.rs#L156).                                                                             |
| **Room Cleanup**          | ✅ **SUCCESS**  | Correctly handled by [unprocman/src/manager.rs](crates/tools/unprocman/src/manager.rs#L95) on exit/timeout.                                                                 |
| **Port Configurability**  | ✅ **SUCCESS**  | Port range and addresses are fully configurable in `.ron` files.                                                                                                            |
| **Room Code Positioning** | ❌ **DEVIATED** | Located at the bottom of the player list ([LobbyMainUI](crates/unlobby-plugin/src/systems/lobby_main.rs#L214)) instead of top-right.                                        |
| **"Copy to Clipboard"**   | ❌ **MISSING**  | No implementation for clipboard interaction or confirmation feedback.                                                                                                       |
| **Hub Status Indicator**  | ❌ **MISSING**  | Main Menu lacks a "Hub Connected" visual state ([mainmenu.rs](crates/unmainmenu-plugin/src/mainmenu.rs#L35)).                                                               |
| **Loading & Spinners**    | ❌ **MISSING**  | `HubStatus.is_pending` is tracked but not rendered to the user.                                                                                                             |
| **Error Feedback**        | ❌ **MISSING**  | Errors like "Room not found" only exist in logs, not in the UI ([ui.rs](crates/unhub-plugin/src/ui.rs#L226)).                                                               |
| **Enter/ESC Support**     | ❌ **MISSING**  | No `KeyCode::Enter` handling for joining; [ESC] does not exit the Hub menu.                                                                                                 |
| **Auto-Reconnect**        | ❌ **MISSING**  | Client detects disconnect but makes no attempt to re-establish the session ([connection.rs](crates/unnet-plugin/src/systems/connection.rs#L1031)).                          |
| **Start Mission Guard**   | ❌ **MISSING**  | Host can click start with no map selected; results in a silent no-op/log warning.                                                                                           |
| **Stack Documentation**   | ⚠️ **PARTIAL**  | Moved to [HOSTING.md](../../HOSTING.md). Covers setup and troubleshooting but many TODOs remain (no prebuilt binaries, hardcoded ports, no systemd files, no upgrade path). |

---

## 2. Detailed Journey Inspection

### 2.1 The "Friday Night Host" (Creating a Room)

- **Success:** Room code generation uses the Safe-Vocal alphabet correctly, avoiding ambiguous characters (0, O, 1, I).
- **Failure:** The "prominent" requirement is missed. The code is placed below the player list, which will be pushed
  further down as more friends join (violating "visible without searching").
- **Risk:** If the Hub is down, clicking "Create Room" results in a silent timeout after 5 seconds of the worker thread
  failing (or immediate error log), but the user sees a frozen or non-responsive UI.

### 2.2 The "Late Joiner" (Joining a Room)

- **Success:** Input is case-insensitive. This is well-handled by the hardcoded key mapping in
  [ui.rs](crates/unhub-plugin/src/ui.rs#L156), which also correctly implements the **Safe-Vocal alphabet** to avoid
  ambiguous characters.
- **Failure:** The user must manually click "Join Room" after typing the code. Pressing `Enter` is the standard
  expectation for form-like inputs.
- **UX Gap:** If the code is mistyped, the Hub returns a 404. The game client handles this by logging "Status: 404 Not
  Found" to the terminal, while the user remains on the Hub screen with no indication of why it didn't work.

### 2.3 The "Community Sysadmin" (Hosting a Hub)

- **Documentation:** ⚠️ **PARTIAL** Hosting guide moved to [HOSTING.md](../../HOSTING.md). Covers architecture, setup,
  config reference, troubleshooting, and production notes — but with many honest TODOs (no prebuilt binaries, hardcoded
  ports, no encryption on ProcMan link, no graceful shutdown, no systemd units tested).
- **Configuration Gap:** The configuration files `hub_config.ron` and `procman_config.ron` are generated with default
  values, but the underlying Rust structs ([HubConfig](crates/tools/unhub/src/state.rs#L17) and
  [ProcManConfig](crates/tools/unprocman/src/config.rs#L7)) still lack inline doc-comments explaining the fields (e.g.,
  `official_server_keys`).
- **Binary Consistency:** The project uses `unhaunter_dedicated` as the hardcoded binary path in `unprocman`'s default
  config, which matches the cargo workspace definition.

### 2.4 The "Solo Explorer" (Testing Connectivity)

- **Failure:** The green dot / "Hub Connected" text is entirely absent. Users have to "click and find out" if the hub is
  reachable.

### 2.5 The "Reconnection" (Resilience)

- **Failure:** The logic in [connection.rs](crates/unnet-plugin/src/systems/connection.rs#L1031) correctly detects a
  host disconnect and pauses the game, but it stops there. The CUJ success criteria requires an auto-reconnect attempt.

---

## 3. Checklist: Missing Affordances

### Visibility & Usability

- [ ] **Copy Confirmation:** Even if copy-paste is added, we need a "Copied!" popup to satisfy the "Confirmation?"
      requirement.
- [ ] **Lobby Code Persistence:** While visible in the Lobby, if the host transitions back to Map Selection, the code
      current disappears because it is part of `LobbyMainUI`.

### Loading & Connection States

- [ ] **Double-click Protection:** Rapidly clicking "Join Room" triggers multiple sequential HTTP requests to the Hub
      worker.
- [ ] **Progress Phase Indication:** The user sees no distinction between "Contacting Hub" and "Waiting for Server
      Allocation".

### Start Mission Flow

- [ ] **Empty Map Guard:** If the host clicks "Start Mission" without a map, it logs `warn!` and does nothing. The user
      should see a "Please select a map" hint.
- [ ] **Joiner Map View:** While joiners can see the map info, there is no visual "Host is selecting a map..." state for
      joiners to know why the lobby is idle.

---

## 4. Backend & Protocol Risks

### Hub Capacity

- The `join_room` API in [api.rs](crates/tools/unhub/src/api.rs#L102) does not check the `player_count` against a
  maximum (e.g., 4 players). A room could technically report a successful join for a 5th player, who would then be
  rejected by the TCP server's internal logic, leading to a confusing "Connection Refused" error.

### Zombie Rooms

- The Hub has a good cleanup mechanism: `state.rooms.retain(|_, room| room.server_id != uuid);` ensures that if a
  ProcMan crashes, its rooms are purged.
- However, if a Dedicated Server hangs during startup but `unprocman` still thinks it's starting, the Hub might keep the
  room entry until the ProcMan's own heartbeat timeout (which isn't strictly enforced in the current loop).

### Version Mismatches

- The Hub checks versions, but the `unhub-plugin` in the game client sends `CARGO_PKG_VERSION` without a "minimum
  version" check on the Hub side beyond string equality.

---

## 5. Sysadmin & Infrastructure

- **Documentation Gap:** While the `unhub` and `unprocman` binaries exist, the configuration structs in
  [state.rs](crates/tools/unhub/src/state.rs) and [config.rs](crates/tools/unprocman/src/config.rs) are currently
  undocumented (no code comments/doc-strings), which slightly misses the "self-documenting" success criterion.
- **Room Cleanup Success:** This is correctly implemented. When a dedicated server process exits (e.g., after the
  5-minute idle timeout in [connection.rs](crates/unnet-plugin/src/systems/connection.rs#L1175)), `unprocman` notifies
  the Hub to remove the room ([manager.rs](crates/tools/unprocman/src/manager.rs#L95)).
