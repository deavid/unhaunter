# Hub Rate Limiting & DoS Mitigation Plan

- **Date:** 2026-02-16
- **Status:** Draft / Design — Questions Resolved
- **Depends on:** `01_proposal.md` (architecture), `05_critical_user_journeys.md` (UX), [HOSTING.md](../../HOSTING.md)

---

## 1. The Problem

The Hub's `POST /v1/rooms/create` endpoint allocates real server resources (a `unhaunter_dedicated` process: ~50-100MB
RAM, ~10% CPU on 2-vCore) with **zero authentication or cost to the caller**. The only "identity" is a self-reported
UUID, which can be generated freely.

A trivial `while true; do curl ...; done` loop — or a modest script sending requests with random UUIDs — can exhaust all
100 ports in the configured port range in seconds. Each room sits idle for 5 minutes before timing out. This is a
denial-of-service attack that requires no sophistication and costs the attacker nothing.

Current defenses: **none.**

---

## 2. Proposed Mitigations

Two complementary mechanisms, each targeting a different dimension of the attack:

### 2.1 Per-IP Active Room Cap

**What:** The Hub tracks which client IP created each active room. A single IP cannot have more than N active rooms
simultaneously.

**Why:** Limits the _total damage_ a single attacker can inflict. Even with unlimited time, one IP can only hold N
server processes hostage.

**Mechanism:**

- Hub maintains a `HashMap<IpAddr, Vec<String>>` (IP → list of active room codes).
- On `create_room`: check if the IP already has N active rooms. If so, reject with `429 Too Many Requests`.
- On room close (timeout, all players left, ProcMan reports `RoomClosed`): remove the room code from the IP's list.
- The map is in-memory, same as the rest of Hub state. No persistence needed.

**IP extraction:** The Hub will typically be behind a reverse proxy (Nginx/Caddy). The real client IP comes from
`X-Forwarded-For` or `X-Real-IP` headers. When there is no proxy, it's the TCP peer address. Controlled by
`trust_proxy_headers` in `hub_config.ron`.

**Resolved decisions:**

1. **N = 2.** If multiple people at the same location are playing, they should join the same room, not create separate
   ones. A cap of 2 allows for one active room + one "oops, forgot to leave" ghost room. A third creation is blocked
   until a room expires or closes.

2. **`join_room` is NOT IP-capped.** Joining is cheap (no server allocation). The room secret provides sufficient
   gating.

3. **`trust_proxy_headers: bool` config flag.** Default `false` (use TCP peer address). Set to `true` when behind a
   reverse proxy. The Hub is designed to be exposed directly in simple deployments and behind Nginx/Caddy in production.

4. **IPv6: per-address (`/128`) for now.** Revisit only if subnet-rotation abuse is observed in practice.

---

### 2.2 Proof-of-Work Client Challenge

**What:** Before creating a room, the client must solve a computational puzzle. The Hub issues a challenge (nonce), the
client finds a value X such that `sha256(nonce || X)` has K leading zero bits, and submits the solution with the create
request. The Hub verifies the solution (O(1), single hash) before proceeding.

**Why:** Makes _flooding_ expensive. Each room creation costs the attacker ~100ms of CPU, making a 1000-room attack take
~100 seconds of dedicated computation instead of <1 second of network requests. Does not affect legitimate users (one
100ms computation before creating a room is imperceptible in a flow that involves typing a room code to friends).

**Mechanism:**

1. Client calls `POST /v1/challenge` with `{ player_uuid }` → Hub returns `{ nonce, difficulty }`.
2. Hub stores the nonce bound to both the `player_uuid` and the requesting IP.
3. Client computes: find `solution` (u64 or hex string) such that `sha256(nonce || solution)` starts with K zero bits.
4. Client calls `POST /v1/rooms/create` with `{ player_uuid, game_version, nonce, solution }`.
5. Hub verifies:
   - Nonce was issued by this Hub (not replayed/fabricated).
   - Nonce has not expired (2-minute TTL).
   - Nonce has not been used before (single-use — consumed on successful `create_room`).
   - Nonce belongs to the `player_uuid` submitting the create request.
   - `sha256(nonce || solution)` has K leading zero bits.
   - If all checks pass, proceed with room creation.

**Nonce ownership and limits:**

- Each nonce is bound to the `player_uuid` that requested it. A nonce issued to UUID A cannot be used by UUID B.
- **Max 2 outstanding nonces per UUID** at any time. A third challenge request for the same UUID is rejected.
- **Max 2 outstanding nonces per IP** at any time. A third challenge request from the same IP is rejected.
- Nonces expire after 2 minutes. With max 2 outstanding, this effectively limits to ~1 new room per minute per IP/UUID
  under sustained use.
- Outstanding nonces are cleaned up periodically (remove expired entries every 30 seconds).

**Nonce storage:** In-memory `HashMap<String, NonceEntry>` where `NonceEntry` contains
`{ player_uuid, client_ip, issued_at }`. Lightweight — even under heavy load, a few thousand entries × ~150 bytes =
sub-MB.

**Why `POST /v1/challenge` (not `GET`)?** The endpoint requires a `player_uuid` in the body to bind the nonce. It also
checks the ban list (no point issuing challenges to banned UUIDs) and the per-UUID/per-IP outstanding nonce limits. This
is a state-mutating operation, not a safe/idempotent read.

**Resolved decisions:**

5. **Difficulty K = 20 bits (~100ms on modern desktop).** Configurable via `hub_config.ron`. The target hardware is
   desktop PCs running Bevy — 100ms is imperceptible in a room-creation flow that already takes seconds. If WASM support
   is added later, difficulty can be lowered per-endpoint or a longer timeout can be granted.

6. **PoW runs synchronously in the Hub worker thread.** The worker is single-threaded and sequential. The player is
   waiting for room creation — they cannot do anything else during this flow. 100ms of extra blocking is invisible.

7. **SHA-256 via the `sha2` crate.** Standard, well-understood, widely used. Added as a dependency to both
   `unhub-client` and `unhub`.

8. **`join_room` does NOT require PoW.** Joining allocates no server resources. The room secret is sufficient gating.

9. **Nonces are NOT IP-bound** (an advanced attacker can trivially rotate IPs). Instead, nonces are **UUID-bound** and
   **single-use with 2-minute expiry**. The per-IP outstanding nonce limit (max 2) provides the IP-based throttle.

10. **The challenge endpoint is rate-limited** via the per-IP and per-UUID outstanding nonce caps (max 2 each). No
    separate rate-limiter middleware is needed — the nonce store itself enforces the limit.

---

### 2.3 Never-Joined Room Fast Expiry

**What:** Rooms where no player has _ever_ connected are reclaimed after 30 seconds instead of the standard 5-minute
idle timeout.

**Why:** The 5-minute idle timeout is designed for rooms where players _left_ (e.g., between missions, bathroom break).
A room that was _allocated but never received a single connection_ is almost certainly either:

- An abandoned creation (player changed their mind).
- An attack (automated room creation to exhaust capacity).

In both cases, holding a dedicated server process for 5 minutes is wasteful.

**Mechanism:** The dedicated server already tracks `player_count`. ProcMan tracks per-server state. The check is: if
`room_code.is_some() && player_count == 0 && time_since_assignment > 30s`, ProcMan kills the process and reports
`RoomClosed` to the Hub.

**Impact on attack surface:** Without fast expiry, an attacker with 1 IP and cap=2 can hold 2 servers for 5 minutes.
With 30-second expiry, those 2 servers are reclaimed in 30 seconds, and the attacker must re-solve PoW to claim them
again. Over 5 minutes, that's 10 PoW solves instead of 1 — and they still only ever hold 2 servers.

---

## 3. Combined Flow

```
Client                                Hub
  │                                     │
  ├─ POST /v1/challenge ───────────────►│  1. Check UUID ban list
  │   { player_uuid }                   │  2. Check per-UUID outstanding nonces (< 2)
  │                                     │  3. Check per-IP outstanding nonces (< 2)
  │◄──── { nonce, difficulty } ─────────┤  4. Generate nonce, store with UUID + IP + timestamp
  │                                     │
  │  [client computes PoW: ~100ms]      │
  │                                     │
  ├─ POST /v1/rooms/create ───────────►│  1. Verify PoW (nonce exists, not expired, UUID matches, hash OK)
  │   { player_uuid, game_version,      │  2. Consume nonce (single-use)
  │     nonce, solution }               │  3. Check per-IP room cap (< 2 active rooms)
  │                                     │  4. Check UUID ban list
  │◄──── { code, addr, secret } ───────┤  5. Find ProcMan with capacity → allocate room
  │                                     │  6. Record room → IP mapping for room cap tracking
```

**Rejection priority:** PoW is verified first (cheapest check — one hash computation). Then IP room cap. Then ban list.
Then capacity. This means even a valid PoW solution is rejected if the IP already has 2 rooms, avoiding wasted server
resources.

---

## 4. Protocol Changes

### New Endpoint

```
POST /v1/challenge
  Request:  { player_uuid: Uuid }
  Response: { nonce: String, difficulty: u32 }
```

### New Structs

```rust
pub struct ChallengeRequest {
    pub player_uuid: Uuid,
}

pub struct ChallengeResponse {
    pub nonce: String,
    pub difficulty: u32,
}
```

### Modified `CreateRoomRequest`

```rust
pub struct CreateRoomRequest {
    pub player_uuid: Uuid,
    pub game_version: String,
    // New fields:
    pub nonce: String,
    pub solution: String,
}
```

### New Hub State

```rust
struct NonceEntry {
    player_uuid: Uuid,
    client_ip: IpAddr,
    issued_at: Instant,
}

// In HubState:
pub nonces: Arc<DashMap<String, NonceEntry>>,           // nonce string → entry
pub rooms_by_ip: Arc<DashMap<IpAddr, Vec<String>>>,     // IP → active room codes
```

### New Error Responses

| HTTP Status | Error Code               | When                                                     |
| ----------- | ------------------------ | -------------------------------------------------------- |
| `429`       | `room_cap_exceeded`      | IP already has 2 active rooms                            |
| `429`       | `challenge_rate_limited` | UUID or IP already has 2 outstanding nonces              |
| `400`       | `invalid_pow`            | PoW solution is wrong, expired, reused, or UUID mismatch |
| `400`       | `missing_pow`            | No nonce/solution provided                               |
| `403`       | `banned`                 | UUID is banned (now also returned from `/challenge`)     |

---

## 5. Crate Impact

| Crate                        | Changes                                                                                                                                     |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `unhub-client` (library)     | Add `ChallengeResponse` struct. Add PoW solver function. Add `sha2` dependency.                                                             |
| `unhub` (binary)             | Add `/v1/challenge` endpoint. Add nonce store. Add PoW verifier. Add IP room tracking. Add `sha2` dependency. Modify `create_room` handler. |
| `unhub-plugin` (game client) | Modify `hub_client.rs` worker to call challenge → solve → create (instead of just create).                                                  |

---

## 6. Configuration Impact

### `hub_config.ron` (new fields)

```ron
(
    // ... existing fields ...
    max_rooms_per_ip: 2,
    pow_difficulty: 20,
    challenge_ttl_seconds: 120,
    max_outstanding_challenges_per_ip: 2,
    max_outstanding_challenges_per_uuid: 2,
    trust_proxy_headers: false,
    never_joined_timeout_seconds: 30,
)
```

| Field                                 | Default | Description                                                                       |
| ------------------------------------- | ------- | --------------------------------------------------------------------------------- |
| `max_rooms_per_ip`                    | `2`     | Maximum active rooms created from a single IP.                                    |
| `pow_difficulty`                      | `20`    | Number of leading zero bits required in PoW hash. ~100ms at 20.                   |
| `challenge_ttl_seconds`               | `120`   | Nonce expiry time. Unsolved/unused nonces are discarded after this.               |
| `max_outstanding_challenges_per_ip`   | `2`     | Max pending (unsolved) nonces per client IP.                                      |
| `max_outstanding_challenges_per_uuid` | `2`     | Max pending (unsolved) nonces per player UUID.                                    |
| `trust_proxy_headers`                 | `false` | If `true`, read client IP from `X-Forwarded-For`/`X-Real-IP` instead of TCP peer. |
| `never_joined_timeout_seconds`        | `30`    | Timeout for rooms where no player ever connected (see §2.3).                      |

---

## 7. Resolved Decisions Summary

| #   | Question                        | Decision                                                                 |
| --- | ------------------------------- | ------------------------------------------------------------------------ |
| 1   | Per-IP room cap N?              | **2.** Multiple players at one location should share a room.             |
| 2   | Cap `join_room` per-IP too?     | **No.** Room secret is sufficient gating.                                |
| 3   | Proxy header trust model?       | **Config flag** `trust_proxy_headers`, default `false`.                  |
| 4   | IPv6 per-address or per-subnet? | **Per-address** (`/128`). Revisit if subnet-rotation abuse observed.     |
| 5   | PoW difficulty (K bits)?        | **20** (~100ms). Configurable.                                           |
| 6   | Client-side PoW threading?      | **Synchronous** in worker thread. Player is waiting, nothing else to do. |
| 7   | Hash function?                  | **SHA-256** via `sha2` crate.                                            |
| 8   | PoW on `join_room`?             | **No.** Joining allocates no server resources.                           |
| 9   | Nonce binding?                  | **UUID-bound** (not IP-bound). Single-use, 2-minute expiry.              |
| 10  | Rate-limit challenge endpoint?  | **Yes**, via per-IP and per-UUID outstanding nonce caps (max 2 each).    |

---

## 8. Threat Analysis

### Attack: Single IP, Scripted Flooding

- **Without mitigations:** Attacker exhausts 100 ports in seconds. All capacity gone for 5 minutes.
- **With mitigations:** Attacker creates 2 rooms (per-IP cap). Each costs ~100ms PoW. After 30 seconds (never-joined
  expiry), rooms are reclaimed. Attacker can repeat but only ever holds 2 of 100 servers. **Neutralized.**

### Attack: Multiple IPs (Botnet / Cloud VMs)

To saturate 100 ports, attacker needs 50 IPs, each creating 2 rooms, each solving PoW. Every 30 seconds, all rooms
expire and must be re-created with fresh PoW. Sustained attack requires 100 PoW solves every 30 seconds across 50
machines. This is:

- Non-trivial to orchestrate.
- Requires real money (cloud VMs) or a botnet.
- Targeting a co-op ghost game's community hub.

**Residual risk accepted.** At this threat level, hosting-level protection (Cloudflare, fail2ban) is the appropriate
response, not application logic.

### Attack: UUID Spoofing

Attacker generates random UUIDs per request. **Does not help** — the per-IP cap is the binding constraint, not the UUID.
UUID controls nonce ownership (which nonce can be used with which create request), but the IP cap limits total rooms.

### Legitimate Use: LAN Party (5 Players, 1 NAT IP)

First 2 players to create rooms succeed. Third is blocked. **Expected and correct behavior** — players at the same
location should join each other's rooms. If all 5 genuinely need separate rooms (unlikely), the Hub operator can
increase `max_rooms_per_ip` in config.

### Legitimate Use: Player Crashes, Room Lingers

Player creates room, connects, crashes. Room has `player_count > 0` briefly, then drops to 0. Standard 5-minute idle
timeout applies (not the 30-second never-joined timeout, because this room _was_ joined). Player relaunches, creates a
second room — uses their second slot (cap=2). If they crash _again_, they wait up to 5 minutes. The crash is the
problem, not the rate limit.

### Legitimate Use: Normal Play Session

Player requests challenge → solves in ~100ms → creates room → friends join. Zero friction. The PoW is invisible inside a
flow that takes seconds (network RTT + server allocation + UI transitions). Player never notices.

---

## 9. Non-Goals (for this plan)

- **Accounts / authentication.** Still no accounts. PoW is a _computational_ cost, not an identity requirement.
- **CAPTCHAs.** Not applicable — the game client is the only consumer of this API, not a browser.
- **Memory-hard PoW (Argon2, scrypt).** SHA-256 is sufficient. GPU solvers are a theoretical concern but not a realistic
  threat for a co-op ghost game's community hub.
- **Global rate limiting (e.g., max 10 rooms/minute total).** This would limit legitimate use during peak times. Per-IP
  limits are more targeted.
- **DDoS protection (network layer).** Volumetric DDoS (bandwidth exhaustion) is out of scope — that's a hosting
  provider / Cloudflare concern, not application-level logic.

## Appendix: Author review and thoughts

Proof of work as described will not help. The attacker just requires a very few IPs with a few machines to complete the
challenges on time and create a denial of service.

This is because:

- We spin the server for 5 minutes if no one joins.
- The Proof of work is worth only 100ms
- They only need to reach a concurrency of 200 to effectively do DoS, because that's our capacity.

5 min / 200 = 1.5s ; they only need to do PoW every 1.5s; so technically they only need the IPs, the PoW is not
requiring them to use even a fraction of a single computer.

We need to change how do we spin servers, how we deal with this.

If, for example, we differentiate between the server states to give them different tick rates, we could make servers use
technically 0% while waiting:

- Room == None: We could use Tick rate = 1 Hz.
- Room == Some:
  - Lobby, 0-1 players: We could use Tick rate = 1 Hz.
  - Lobby, 2+ players: We could use Tick rate = 5 Hz.
  - Mission, 1 player:  We could use Tick rate = 15 Hz.
  - Mission, 2+ players:  We could use Tick rate = 60 Hz.

But forget about all these tick rates, the most important thing is that we can have something like 5 servers idle in
Room = None state, permanently. Then when someone wants to create a room, one of these gets the room code.

The trick is, if the player does not connect in time, or leaves - we can be way more aggressive because instead of
stopping the server, we can just erase the room, so Room = None again. We can do that in 5 seconds, probably less.

Also, with this, we could have all our capacity booted at the beginning if we wanted. For example say, we want 200
dedicated max? we could ramp them up on boot. (Or a fraction, because it sounds expensive anyway, but the point is that
they would now take 1/60 of the server's CPU on idle state). Then it's all about juggling the room codes fast enough.

-----

Another thing, the ProcMan. If we don't reserve the room until the client is actually connected we can be much more
aggressive with the timings. For this, ProcMan would need to tunnel the connections itself - so clients would be advertised
to connect to ProcMan which would attempt to fake a dedicated server.

Once the client is there, and knowing the room code, then ProcMan can send them to an available server for real - and
tunnel / relay the connection.

This would prevent attacks that simply hit the REST API. They would need to do the second connection. And once the
client is connected, we can say that we want a heartbeat or something every second - or a constant data transfer or
something - and if that's not the case we could free the room straight away.
