# Hub Security MVP Implementation Plan

**Date:** 2026-02-21

**Status:** Approved Plan

**Goal:** Secure the Hub MVP against basic DoS attacks (RAM exhaustion, OOM crashes) using simple, brutal friction,
without over-engineering.

## 1. Current State Assessment

Based on an audit of the current codebase:

- **Fast Expiry:** ✅ **Already implemented** in `unprocman` (`manager.rs`). Rooms that are never joined are reclaimed
  after 10 seconds. Rooms that were joined are reclaimed after 5 minutes of being empty.
- **Bounded Codecs:** ❌ **Missing**. Both `unhub` and `unprocman` use `LinesCodec::new()` without a max length, making
  them vulnerable to OOM crashes from malformed packets.
- **IP Caps:** ❌ **Missing**. `unhub` does not track or limit room creation by IP address.
- **Proof-of-Work (PoW):** ❌ **Missing**. `unhub` does not require any computational effort to create a room.

## 2. Step 1: Bounded Codecs (OOM Prevention)

**Objective:** Prevent a single attacker from causing an Out-Of-Memory (OOM) crash by sending an infinite string without
a newline over the TCP connection.

**Required Changes:**

1. **`crates/tools/unhub/src/procman.rs`**:
   - Replace `LinesCodec::new()` with `LinesCodec::new_with_max_length(8192)` (or a similarly safe limit like 65536).
2. **`crates/tools/unprocman/src/hub_comm.rs`**:
   - Replace `LinesCodec::new()` with `LinesCodec::new_with_max_length(8192)`.

## 3. Step 2: Per-IP Room Caps (Blast Radius Limiting)

**Objective:** Limit the number of active rooms a single IP can hold to 2. This ensures that even if an attacker
bypasses other limits, they can only hold 2 server processes hostage at a time.

**Required Changes:**

1. **`crates/tools/unhub/src/state.rs`**:
   - Add `pub rooms_by_ip: Arc<DashMap<std::net::IpAddr, Vec<String>>>` to `HubState`.
   - Add `pub room_to_ip: Arc<DashMap<String, std::net::IpAddr>>` to easily reverse-lookup which IP owns a room when it
     closes.
2. **`crates/tools/unhub/src/api.rs`**:
   - Update `create_room` to extract the client's IP address. Use `axum::extract::ConnectInfo<std::net::SocketAddr>`
     (and optionally check `X-Forwarded-For` if `trust_proxy_headers` is enabled).
   - Check if the IP already has `>= max_rooms_per_ip` (default 2) active rooms. If so, return `429 Too Many Requests`.
   - On successful room creation, add the room code to the IP's list in `rooms_by_ip` and map it in `room_to_ip`.
3. **`crates/tools/unhub/src/procman.rs`**:
   - Update the `ProcManMessage::RoomClosed` handler. When a room closes, look up its IP in `room_to_ip`, remove it from
     `rooms_by_ip`, and clean up `room_to_ip`.

## 4. Step 3: Proof-of-Work (PoW) Challenge (Economic Friction)

**Objective:** Make room creation computationally expensive for attackers (~100ms) while remaining imperceptible to
legitimate players.

**Required Changes:**

1. **`crates/unhub-client/src/protocol.rs`**:
   - Add `ChallengeRequest { pub player_uuid: Uuid }`.
   - Add `ChallengeResponse { pub nonce: String, pub difficulty: u32 }`.
   - Update `CreateRoomRequest` to include `pub nonce: String` and `pub solution: String`.
2. **`crates/unhub-client/src/lib.rs`**:
   - Add the `sha2` crate as a dependency.
   - Add a `solve_pow(nonce: &str, difficulty: u32) -> String` function that finds a solution where
     `sha256(nonce || solution)` has `difficulty` leading zero bits.
3. **`crates/tools/unhub/src/state.rs`**:
   - Add `pub nonces: Arc<DashMap<String, NonceEntry>>` to `HubState`.
   - Define
     `pub struct NonceEntry { pub player_uuid: Uuid, pub client_ip: std::net::IpAddr, pub issued_at: std::time::Instant }`.
4. **`crates/tools/unhub/src/api.rs`**:
   - Add a `POST /v1/challenge` endpoint. It should generate a random nonce, store it in `state.nonces`, and return it.
   - Enforce a limit of max 2 outstanding nonces per UUID and per IP to prevent memory exhaustion in the `nonces` map.
   - Update `create_room` to verify the PoW solution before proceeding. Remove the nonce from the map after a successful
     check (single-use).
5. **`crates/unhub-plugin/src/hub_client.rs`**:
   - Update the `HubRequest::CreateRoom` handler. It must first call `/v1/challenge`, run `solve_pow` (which is fine to
     block the worker thread since it's dedicated to Hub communication), and then call `/v1/rooms/create` with the
     solution.

## 5. Step 4: Configuration Updates

**Objective:** Make the security parameters configurable without recompiling.

**Required Changes:**

1. **`crates/tools/unhub/src/state.rs` (`HubConfig`)**:
   - Add `pub max_rooms_per_ip: usize` (default: 2).
   - Add `pub pow_difficulty: u32` (default: 20).
   - Add `pub trust_proxy_headers: bool` (default: false).
2. **`crates/tools/unhub/src/config.rs`** (or wherever config is loaded):
   - Ensure these new fields are parsed correctly with fallbacks.

---

**Execution Note:** These steps are designed to be implemented incrementally. You can start with Step 1 (Bounded Codecs)
as it is a trivial 2-line change, then move to Step 2 (IP Caps), and finally Step 3 (PoW).
