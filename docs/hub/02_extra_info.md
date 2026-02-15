# Hub Service: Extra Design Notes & Rationale

This document captures the detailed thought process, research, and alternative options discussed during the design of
the Hub Service (v1). It serves as a companion to `01_proposal.md`, providing the "Why" behind the "What".

## 1. The "Ultra-Lean" Constraint

The primary architectural driver is the requirement to support **10,000+ concurrent players** on a **$1/month VPS**
(typically 1 vCPU, 512MB RAM).

### 1.1 Why In-Memory State?

Traditional web architectures use a stateless web server + a database (Postgres/MySQL).

- **Bottleneck:** For 10k players, checking "does this room exist?" or "update heartbeat" hits the disk/DB thousands of
  times per second. Even with caching, the serialization/deserialization overhead is significant for a low-spec CPU.
- **Solution:** `DashMap` (Concurrent Hash Map) in RAM.
- **Cost:** A room entry is small (~200 bytes). 10,000 rooms ≈ 2MB of RAM. This fits easily in 512MB, leaving plenty for
  the OS and application logic.
- **Result:** O(1) lookups, sub-millisecond latency, zero disk I/O for hot paths.

### 1.2 Why RON/JSON over SQLite?

We considered SQLite but opted for RON (Rusty Object Notation) or JSON for persistence.

| Feature         | SQLite                                  | RON/JSON (File-based)                  | Decision                                                                                         |
| :-------------- | :-------------------------------------- | :------------------------------------- | :----------------------------------------------------------------------------------------------- |
| **Overhead**    | Low, but non-zero (engine, SQL parsing) | Zero (direct struct serialization)     | **RON** wins for "lean" goal.                                                                    |
| **Concurrency** | Excellent (WAL mode)                    | Poor (Global lock on write)            | **RON** is acceptable because persistence is _rare_ (config changes, bans). Hot state is in RAM. |
| **Migrations**  | SQL Scripts (up/down)                   | Code-first (Load -> Transform -> Save) | **RON** simplifies deployment (no `db migrate` step).                                            |
| **Admin**       | Requires SQL knowledge / tools          | Text editor                            | **RON** is friendlier for small-scale admins.                                                    |

**Migration Strategy for RON:** Since there is no DB engine to enforce schema, we handle it in code:

1. Load `config.ron`.
2. If `version < current`, run Rust transformation logic.
3. Save `config.ron` with new structure.
4. Keep `.bak` files for safety.

## 2. Linguistic Design: The "Safe-Vocal" Code

The room code system is based on research into **Phonetic Distinctness** and **Miller's Law** (Chunking).

### 2.1 The Problem with Random Strings

- **Ambiguity:** "B" vs "P", "M" vs "N", "S" vs "F" are hard to distinguish over voice chat (especially with
  compression).
- **Offensiveness:** Random vowels can accidentally spell slurs.
- **Cognitive Load:** `XJ94L2` is hard to remember. `K7-WR-P` is easier.

### 2.2 The "Safe-Vocal" Alphabet Selection

We selected a Base-20 set: `C, D, F, G, H, J, K, L, M, P, R, S, T, V, W, X, 2, 4, 7, 9`.

**Exclusions & Reasoning:**

- **Vowels (A, E, I, O, U, Y):** Removed to prevent accidental word formation.
- **The "E" Group (B, C, D, E, G, P, T, V, Z):** Many of these rhyme. We kept some (C, D, G, P, T, V) to maintain
  entropy but removed the most confusable ones like B and Z.
- **Visual Ambiguity:**
  - `1`, `I`, `l` (One, Eye, Lima) -> Removed.
  - `0`, `O`, `Q` (Zero, Oscar, Quebec) -> Removed.
- **Phonetic Ambiguity:**
  - `M` vs `N`: Kept `M`, removed `N`.
  - `S` vs `F`: Kept both, but they are distinct enough in most accents.
  - `5` vs `S`: Removed `5`.
  - `6` vs `X`: Removed `6`.
  - `8` vs `H`: Removed `8`.

**Fallback Option (The "Ultra-Clear 12"):** If the Base-20 set proves too noisy in testing, we can fall back to this
stricter Base-12 set: `D, F, H, J, K, L, P, R, S, W, X, Z`. (Entropy: $12^5 = 248,832$ combinations, still sufficient
for <10k concurrent rooms).

## 3. Architecture & Component Decisions

### 3.1 Why "ProcMan" (Process Manager)?

Initially, we considered having the Hub spawn processes directly. We moved to a split architecture (`unhub` +
`unprocman`) for:

1. **Security:** The Hub is public-facing. If compromised, it shouldn't have shell access to spawn processes.
2. **Scalability:** We can run one Hub and multiple "Game Nodes" (VPS instances running `unprocman`).
3. **Networking:** `unprocman` connects _outbound_ to the Hub. This bypasses the need for complex firewall/NAT
   configuration on the game nodes.

### 3.2 The "Secret" Token Flow

To prevent players from bypassing the Hub (e.g., port scanning to find open rooms), we introduced a **Room Secret**.

- **Mechanism:** `Hub` generates a random string -> sends to `ProcMan` -> `ProcMan` launches `undedicated` with secret.
- **Validation:** Player gets secret from Hub API -> sends to `undedicated` in handshake.
- **Security Level:** Low/Medium. It prevents casual scanning. It does _not_ prevent a player from sharing the secret
  with a friend (which is fine/intended).
- **Alternative Considered:** Cryptographic tokens (JWTs) signed by the Hub.
  - **Verdict:** Overkill for v1. A simple string match is faster and easier to debug.

## 4. Discarded Ideas (for v1)

### 4.1 The Public Server Browser

**Idea:** List all open rooms so anyone can join. **Reason for Rejection:**

- **Moderation Nightmare:** Public rooms require kick/ban tools, profanity filters, and active moderation.
- **Privacy:** Unhaunter is currently designed as a "play with friends" experience.
- **Technical Complexity:** Requires real-time list updates, pagination, and filtering. **Status:** Deferred to Phase 2.

### 4.2 Free-Text Nicknames

**Idea:** Let players type their name. **Reason for Rejection:**

- **Profanity:** Requires complex filtering libraries.
- **UI:** Requires text input handling (IME support, focus management) which is tricky in game engines.
- **Solution:** "Codename" system (Random Adjective + Noun). Fun, thematic, and safe.

### 4.3 WebAssembly (WASM) Multiplayer

**Idea:** Play the game in the browser. **Reason for Rejection:**

- **Transport Protocol:** Browsers cannot do raw TCP. We would need WebSockets or WebRTC.
- **Bevy Support:** `bevy_renet` or `unnet` would need a transport adapter switch.
- **Status:** Deferred. The Hub API is HTTPS-ready, but the game transport is the blocker.

## 5. Future Considerations (Phase 2+)

- **Community Hubs:** The architecture allows anyone to run a Hub. We could federate them or allow the client to choose
  a "Universe".
- **Metric Collection:** The Hub is a perfect place to collect anonymous stats (mission duration, win/loss rates) to
  balance the game.
- **The "Mixer":** A true matchmaking queue that groups solo players.
- **TLS/mTLS:** Upgrading the `ProcMan` <-> `Hub` connection to use mutual TLS for security across untrusted networks.
