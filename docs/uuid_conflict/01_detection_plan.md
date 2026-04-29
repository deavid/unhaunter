# UUID Conflict Detection Plan

## Goal
Detect when two instances of the game are running simultaneously with the same `installation_id` (UUID) and report this conflict to the player.

## Proposed Changes

### 1. `unhub-client` (Protocol)
- **`PingRequest`**: Add `session_id: u16`.
- **`MultiplayerStatus`**: Add `Conflict` variant.

### 2. `unhub-plugin` (Bevy Client)
- **`HubClient` Resource**: Add `session_id: u16` field.
- **`setup_hub_client`**: Generate a random `u16` using `rand::thread_rng().gen()` and store it in `HubClient`.
- **`ping_hub_system`**: Include the `session_id` in the `PingRequest`.
- **`update_hub_status`**: Detect `MultiplayerStatus::Conflict` and update a UI-accessible status (to be displayed by the game's UI).

### 3. `unhub` (Hub Server)
- **`HubState`**:
    - Add `active_sessions: Cache<Uuid, u16>` to track session IDs (TTL: 10m).
    - Add `player_to_room: DashMap<Uuid, String>` to track active lobby membership.
- **`ProcMan` Handler**: Update `player_to_room` on `PlayerJoined` and `PlayerLeft` messages.
- **`ping` handler**:
    - If `installation_id` has a different `session_id` AND is in `player_to_room`, return `MultiplayerStatus::Conflict`.
    - This allows sequential runs and multiple instances at the main menu, but flags conflicts if one is actively in a lobby.
- **`create_room` / `join_room` handlers**:
    - Reject if `player_uuid` is already in `player_to_room`.

## Considerations
- **TTL**: The `active_players` cache has a TTL of 2 hours. This might be too long for conflict detection (e.g., if a game crashes and is restarted, the old session ID might persist in the cache for up to 2 hours). We should probably reduce the TTL for "active" status or use a separate shorter-lived cache for session validation.
- **Lobby tracking**: While we could track player-to-room mappings, the `session_id` approach is more general and detects conflicts even before joining a lobby.
