# Multiplayer Architecture Tensions — 2026-02-14

A record of a sprawling conversation about the hardest open decisions in Unhaunter's multiplayer infrastructure. This
document captures the journey — the worries, the tensions, the things that feel stuck — not just conclusions. No code
was written. Some clarity was gained; full resolution was not reached.

This continues from the 2026-02-09 multiplayer vision discussion, which covered the initial design, phased roadmap,
lobby vision, identity model, and the original relay-vs-dedicated-server analysis. That document ended with the
dedicated server as the chosen next step. This conversation reopens and deepens that decision.

---

## Context: What Has Been Built (as of 2026-02-14)

### Technical Readiness

Unhaunter has transitioned from a 1:1 CLI prototype to a **multi-client, host-authoritative architecture**:

- **Multiplayer:** Supports up to 9 players (1 host + 8 clients). Functional **Lobby** where players gather, select maps
  and difficulty, and see each other's status before starting a mission.
- **Dedicated Server:** A headless `unhaunter-server` binary exists. Handles the game simulation (ghost AI, state
  transitions, health) without requiring a GPU, window, or audio device.
- **Networking:** Custom TCP + JSONL protocol with heartbeats, late-join support, and "disconnect protection" (where
  disconnected player entities automatically hide from the ghost).

### Recent Development (Plans 21-27)

- **Architectural Refactoring:** Split the massive networking "god file" into specialized modules (transport, protocol,
  host/client sync).
- **Thin Server Optimization:** Analyzed and removed heavy environmental simulations (thermal, light, sound, fog grids)
  from the server. These are now computed locally by clients to save server CPU.
- **Lobby Polish:** Rebuilt the lobby UI using standard templates to support mouse interaction and visual consistency.
- **Authority Refinement:** Clearly separated "Simulation Authority" (the server) from the "Room Owner" (the player who
  controls the lobby).

### Known Issues Coming In

- **Architectural Debt:** System built by "design by accretion." The snapshot message has grown to 22+ fields. State is
  sometimes triplicated (e.g., item ownership tracked in three different places), leading to fragile sync.
- **The "Thin Server" Risk:** Pushing grids to the client introduces a "sanity circularity" problem where the server
  must trust client-reported sanity values to drive ghost rage — potential vector for desyncs or cheating.
- **UX Friction from Playtests:** "Bouncy" gear interactions, ghost animations not syncing during hunts, "van entry"
  logic occasionally affecting the wrong player.
- **The 5-Player Problem:** If only a few players exist worldwide, traditional matchmaking fails.

---

## The Author's Mindset Coming Into This Conversation

The author is "quite jumbled." The word that keeps coming up is **stuck**. Not on any single technical question, but on
a tangle of interconnected decisions where each choice constrains others. The feeling is of decision paralysis — too
many options on the table, each with real tradeoffs, and an inability to commit to a direction.

### What the Author Cares About (Priorities)

1. **Frictionless experience for players.** No port forwarding, no IP sharing, no networking knowledge required. The
   main UI should show rooms or accept room codes, not ask for IP addresses. The `--host`/`--join` CLI flags are fine
   for power users and off-grid play, but the default experience should be seamless.

2. **Sustainability as a FOSS project.** The author is a solo developer. Infrastructure costs must be low and
   predictable. Paying to run game servers for others to play is a concern. The game must not become a financial burden
   to maintain.

3. **Community bootstrapping.** The game needs a community to exist quickly. Players need to find each other. An empty
   server browser is worse than no multiplayer. The "5-player problem" is always in the background.

4. **Shipping something braggable soon.** There is timeline pressure — the author wants to get multiplayer into players'
   hands in a state worth showing off. Not "someday" but "soon."

5. **The game as a "place."** Long-term, Unhaunter should feel like a persistent social space, not just a program you
   launch to do a mission. The lobby is envisioned as an in-game physical space, potentially the player's "house."

### What the Author Does NOT Care About (or Explicitly Deprioritizes)

1. **Cheating prevention.** This is a co-op game, not competitive PvP. "One only cheats to oneself. And annoys others."
   Server-authoritative models are not justified by anti-cheat needs. If leaderboards eventually exist, moderation tools
   handle abuse — not architectural enforcement.

2. **Premature architectural perfection.** The 22-field snapshot, the tripled state — the author acknowledges these are
   AI-generated concerns that got amplified. The author is the one who chose JSONL over TCP and "trust the client,
   figure it out later." These are pragmatic and intentional, not tech debt born of ignorance.

3. **Accounts.** Accounts are friction. The game should not require signup, email, or passwords. Identity based on
   installation UUID (and eventually keypairs) is preferred. "Accounts are friction" is stated as a hard constraint.

4. **Voice chat (for now).** Critical for the long-term vision (especially public rooms with strangers), but explicitly
   too large a scope item to tackle now. Friends use Discord. Strangers can wait.

### What DOES Worry the Author

1. **Trolls and bad actors.** Not cheaters — trolls. People who join to ruin others' experience. The author is "not
   super confident" that UUID-based banning is sufficient. Community moderation tools are seen as the real solution, but
   the specifics are unclear.

2. **Relay abuse.** If a relay is deployed on the public internet, how do you prevent malicious actors from using it as
   a general-purpose TCP proxy? This is a concrete, unsolved worry.

3. **Server CPU cost.** The Bevy scheduler's multithreaded overhead is measurable even when idle. On a 2-vCore VPS, the
   headless server consumes ~10% CPU doing nothing. This is a fixed cost per room that limits scalability.

4. **Supporting too many modes.** If the dedicated server stays, the relay stays, the host mode stays, and the discovery
   service is added — that's a lot of infrastructure and code paths to maintain as a solo developer.

5. **Picking the wrong direction.** Time invested in the dedicated server might be wasted if the relay model is
   ultimately the better fit. Conversely, pivoting to relays might throw away working dedicated server code.

---

## The Central Decision: Relay (a) vs. Bevy Headless (b)

This is where the conversation spent the most time. The author explicitly called out that these are "wildly different
products with completely different limitations" and that the distinction was being blurred.

### Option (a): Tokio Relay

A standalone service — no Bevy, no game code. Built with Tokio (or similar async runtime). Pure TCP pipe forwarding
between a host player and client players.

**Characteristics:**

- A single binary can handle hundreds or thousands of rooms simultaneously.
- Does not run game code. Does not understand the game protocol (or understands it minimally).
- The host player's machine runs the full game simulation. Clients connect to the host through the relay.
- The relay solves NAT traversal: all connections are outbound from both host and client, going through the relay.
- Abuse prevention is hard because the relay is protocol-unaware — it's just forwarding bytes.
- Latency path: Client → Relay → Host → Relay → Client (double hop).

**The host problem:** Someone's machine must be the authority. If that player disconnects, the game dies. If their
machine is weak, everyone suffers. If they're on WASM, they can't host at all. The relay doesn't solve "who hosts."

### Option (b): Bevy Headless Dedicated Server

A full (but lean) game process running Bevy in headless mode. Understands the game, acts as the center of the star
topology.

**Characteristics:**

- Currently limited to one room per process (multi-room is unexplored territory).
- Incurs fixed overhead from the Bevy scheduler, even when idle.
- Runs game code: ghost AI, state transitions, arbitration. Can validate what clients send.
- Is the central authority — no player needs to "host." All players are equal clients.
- Latency path: Client → Server → Client (single hop, server IS the authority).
- Abuse prevention is natural — it only processes valid game traffic.
- Harder to scale: fixed per-room cost vs. the relay's near-zero per-room cost.

### The Phasmophobia Precedent

Research into Phasmophobia's architecture revealed that it uses the relay model (host ← relay → client), not dedicated
game servers. Their infrastructure handles player data, matchmaking, and room coordination — not game simulation. The
games themselves run on player machines.

This was significant to the author: the most successful game in the genre doesn't use dedicated game servers, and the
added relay latency is meaningless for a game about reading thermometers, not twitch shooting.

### The Tension

The author already built Option (b). It works. But the scalability concern is real, and the Phasmophobia precedent
suggests Option (a) might be the industry-proven approach.

However, Option (b) has advantages that Option (a) cannot match:

- WASM clients can't be hosts, so Option (a) requires a native host player or falls back to... a dedicated server.
- The in-game lobby vision (lobby as a Bevy game space) requires Bevy to run somewhere.
- Testing is easier with a dedicated server (one window to manage, not two).
- The author's long-term simulation ambitions (flow dynamics, heavy environmental computation) may eventually need
  server-side compute.

The author's key observation: **"If the Bevy headless can be made efficient enough within expected metrics, then a relay
isn't even needed."**

---

## Server Capacity Analysis

The author did concrete analysis on dedicated server resource usage:

### Current Measurements (Old 2-vCore VPS)

- **Idle:** ~10% CPU (Bevy scheduler overhead, no players, no simulation).
- **Active (players connected):** ~15% CPU.
- **Total capacity (2 vCores = 200%):** ~13 rooms.

### Optimization Levers Identified

1. **Single-threaded mode:** Dropping the multithreaded Bevy scheduler reduces idle cost from ~10% to ~2.5% (4×
   improvement).
2. **Lower tick rate:** Server doesn't need 60Hz. 20Hz (or even lower for lobby) is sufficient. Systems can run in
   `FixedUpdate` at reduced frequency while positions interpolate smoothly each frame.
3. **Multi-room per process:** Unexplored but potentially significant. If Bevy's `World`/`SubApp` architecture allows
   multiple independent simulations in one process, the scheduler overhead is shared across rooms.

### Projected Capacity (New 8-vCore VPS, ~€10/month)

- 8 vCores with ~50% faster per-core performance vs. old VPS = ~12× total capacity.
- With optimizations: ~20 rooms (optimized) × 12 (hardware) = **~240 rooms.**
- **The author's reaction:** "And suddenly, that sounds like headroom. Like, no way we're getting 240 rooms in parallel
  ever. If we get that, probably we can think on offloading to the community to provide their servers."

### Implication

The capacity concern may be solved by pragmatic optimization + slightly better hardware. 240 rooms is far beyond what a
small FOSS game would need. Even half that is generous. This shifts the calculus toward Option (b) — if the numbers work
out, the relay's scaling advantage becomes irrelevant because the dedicated server scales "enough."

---

## The Discovery Service ("Hub")

Both models (relay and dedicated server) need a discovery service. This is agreed upon and not contentious. The author
describes it as:

> "A proper server/daemon that handles social, handles rooms, and all the light logic such that it can handle thousands
> of clients at the same time, something very lightweight to run all the social platform."

### What the Hub Does

- Room registry (create, list, join by code)
- Player identity (UUID-based, no accounts)
- Ban lists / moderation
- Server directory (for community-run servers)
- "Light logic" — social platform features, presence

### What the Hub Does NOT Do

- Run game simulations
- Understand game mechanics
- Require Bevy or any game engine dependencies

### Implementation

A small Rust web service (axum, SQLite). One instance handles thousands of players. Runs on the VPS. It is the
"unhaunter.com" backend that was envisioned in earlier documents.

### The Author's Urgency

> "This is something I'm going to need very very soon... because the other parts of the multiplayer work, and this is
> missing for 'the experience.'"

The hub is not a "Phase 4 someday" item. It is the missing piece that turns working multiplayer into a shippable
product. Without it, multiplayer requires sharing IP addresses — which is the exact friction the author wants to
eliminate.

---

## The Lobby Boundary Question

This question emerged during discussion and created additional confusion:

> "When we enter a room code or host a room... we join the Bevy headless server?"

### The Problem

The lobby is envisioned as an in-game space — a Bevy game world with player movement, interaction, and eventually
physical presence. If the lobby is a Bevy simulation, it needs to run _somewhere_. Options:

1. **Lobby runs on the dedicated server:** Players connect to the Bevy headless server to enter the lobby. The server
   runs the lobby simulation. This is how it currently works. But it means idle lobby rooms consume Bevy scheduler
   overhead even when nobody is in a mission.

2. **Lobby is client-local:** The lobby runs purely on the player's machine as a local space (the "personal house"
   vision). Players see each other via lightweight presence data from the hub. When they're ready to play, a dedicated
   server instance is spun up for the mission only.

3. **Lobby is hosted by a player:** The host player's machine runs the lobby simulation. Other players connect to the
   host through the relay or directly. This is the host-client model applied to lobbies.

### Tensions

- If lobbies run on the dedicated server, idle rooms burn CPU. But the future vision is "lobby as a game space" which
  requires full Bevy simulation.
- If lobbies are client-local, the "see each other in the lobby" social experience requires a separate lightweight sync
  mechanism (not the full game protocol).
- If lobbies are host-player-run, we're back to the relay model's "someone has to host" problem.
- If solo play is "secretly multiplayer" (with others able to join), then even a solo session has a lobby, and that
  lobby needs to be reachable.

### The Author's Instinct

The author leans toward lobbies on the dedicated server because:

- It already works that way.
- The in-game lobby vision requires Bevy to simulate the space.
- Separating lobby from mission would risk breaking the host-client mode.
- A lobby with no ghost, no grids, no complex simulation should be extremely cheap to run.

### Mitigation: Adaptive Tick Rate

A lobby room doesn't need 60Hz updates. Dropping to 5-10Hz when in lobby mode would make the cost negligible. Nobody
needs 60Hz fidelity for "player walked to the whiteboard." The server can ramp up to full tick rate when a mission
starts and drop back down when it ends.

### Dedicated Server vs. Lobby Scope

The author drew an important distinction:

> "The dedicated server — `unhaunter_dedicated` — I think this one should not handle the lobby. I think this one should
> jump straight to mission."

This implies a possible split:

- **Hub + lobby handling:** Social, rooms, lobby simulation (lightweight).
- **Dedicated server:** Mission simulation only (heavier, spun up on demand).

But this creates its own complexity — now there are two different server processes, and players must be handed off from
one to the other at mission start. The author didn't resolve this and remains uncertain.

---

## Ideas Explored But Not Committed To

### The "Mixer" / Streaming Event Mode

Instead of searching for public rooms, all players who opt in get mashed together into random groups. Each group does
one mission, then players are reshuffled.

**Motivation:**

- Good for Twitch streams — invite viewers to play, create chaos, showcase the game.
- Solves the cold-start problem — no empty room anxiety, just "press button, get group."
- Lets strangers meet in a low-commitment way.

**The author's reaction:** "I like this, it's worth writing down." Not a near-term priority, but a feature that could
drive community events and content creation.

### Solo Play as Secretly Multiplayer

All solo play is actually a multiplayer session with one player. If the player opts in, their room can appear in a
public browser and strangers can late-join.

**Motivation:**

- Blurs the single/multi boundary (a core design goal).
- Creates "spontaneous co-op" moments.
- Players never experience the "searching for a game and finding no one" rejection.

**Tensions:**

- Trolls joining unwanted. Needs an easy "set to private" toggle.
- If on a dedicated server, this means every solo session occupies a server room. If the server has 240-room capacity,
  this could fill up if the game gets popular.
- If client-hosted, WASM players can't participate.

**The author's engagement:** "That's where I was going." This is a deeply held goal, not a passing idea.

### Community Server Ecosystem

Like Minecraft — let communities run their own `unhaunter-server` instances. The hub maintains a server directory.
Players browse and join community servers.

**The author's interest:**

> "This is something that I definitely want. I have a few ideas to eventually have some allowed way or official way to
> add servers, or even server list providers."

This is a strong commitment. The community server model is seen as the long-term scalability answer — if the game gets
popular enough to exceed the author's infrastructure capacity, communities absorb the compute cost.

### Scheduled Community Events

Structured play sessions (" Every Saturday at 8pm UTC, we play together"). Could be supported in-game with
announcements, countdowns, and auto-room-creation.

**Context:** Connected to the Twitch streaming idea. A streamer hosts a session, invites viewers, plays for a few hours.
Good for community building and content. The mixer mode would enhance this.

---

## The Deep Worry: Supporting Too Many Modes

An implicit thread throughout the conversation: the author is a solo developer maintaining multiple connection modes,
server configurations, and deployment scenarios:

1. **Solo play** (local, no networking).
2. **Host + client** (`--host`/`--join`, direct TCP connection, one player hosts).
3. **Dedicated server** (headless Bevy process on a VPS, all players are clients).
4. **Discovery service / hub** (room codes, server browser, social features — separate non-Bevy service).
5. **Relay** (for NAT traversal in host+client mode — if adopted).
6. **Community servers** (third-party-hosted dedicated instances, registered with the hub).

Each mode is simple individually, but the combinatorial surface is large:

- Does the lobby work the same way in host mode and dedicated server mode?
- When a solo player "opens their room," do they become a host, or does a dedicated server instance get allocated?
- Does the hub need to know the difference between a player-hosted game and a dedicated server game?
- Can a WASM client play in all modes, or only some?

The author hasn't explicitly named this worry, but it pervades the conversation. Every time a clean answer emerges for
one scenario, a follow-up question arises about how it interacts with another scenario.

### The Underlying Fear

> "If I don't drop [the dedicated server], I have to support it. And that means additional codepaths and stuff."

The fear is not about any single piece being too complex. It's about the total maintenance burden of supporting all
these modes simultaneously, as a solo developer, while also building the actual game.

---

## What We're Converging Toward (Not Decided, But Emerging)

Based on the conversation, a fuzzy picture is forming:

### Architecture Likely Shape

```text
┌──────────────────────────────────────────────────────┐
│  Hub / Discovery Service                             │
│  (Non-Bevy. Tokio + axum + SQLite.)                  │
│  - Room registry, room codes, player UUIDs           │
│  - Server directory (community + official)           │
│  - Ban lists, moderation                             │
│  - Player presence, social features                  │
│  - Lightweight — handles thousands of clients        │
│  - Knows nothing about game state                    │
│  - Runs on author's VPS                              │
└──────────────────┬───────────────────────────────────┘
                   │ "Room ABC is on server X at ip:port"
                   ▼
┌──────────────────────────────────────────────────────┐
│  Game Server (Bevy Headless)                         │
│  - Runs one or more rooms                            │
│  - Each room is "lobby" or "in mission"              │
│  - Thin arbiter: ghost AI decisions, state machine,  │
│    item ownership, mission phase transitions         │
│  - Does NOT compute grids (thermal, light, sound,    │
│    fog) — clients do this locally                    │
│  - Adaptive tick rate (5Hz lobby, 20-60Hz mission)   │
│  - Can be:                                           │
│    - Author's official server on the VPS             │
│    - Community-hosted on someone else's machine      │
│    - Embedded in a player's client (host mode)       │
│  - Same arbiter code in all three deployments        │
└──────────────────────────────────────────────────────┘
           ▲                           ▲
           │                           │
     ┌─────┘                           └─────┐
     │                                       │
┌────┴──────────────┐             ┌──────────┴────────┐
│  Native Client    │             │  WASM Client      │
│  - Full sim       │             │  - Client-only    │
│  - Can embed the  │             │  - Connects via   │
│    arbiter (host) │             │    WebSocket      │
│  - Direct connect │             │  - Cannot host    │
│    or via hub     │             │                   │
└───────────────────┘             └───────────────────┘
```

### Key Principles Emerging

1. **The dedicated server stays, but its role is "thin arbiter."** It runs ghost AI decision-making and state
   arbitration, not environmental simulation. It's lean enough to run 200+ rooms on a modest VPS.

2. **Host mode is the arbiter embedded in a client.** Same code path for the arbiter logic, different deployment. This
   minimizes the "extra code paths" concern.

3. **The hub is a separate, non-game service.** It handles everything social and logistical. It's the piece that makes
   multiplayer feel frictionless (room codes instead of IPs, public room browser, etc.).

4. **The relay question is deferred.** If the dedicated server model works at the expected scale, relays are
   unnecessary. The server IS the relay — clients connect to it, and it routes messages. If host mode eventually needs
   NAT traversal without a dedicated server, a relay can be added later as a simple TCP proxy coordinated by the hub.

5. **`--host` and `--join` stay forever.** Off-grid, infrastructure-less direct play. Not the default experience, but
   always available. This is a FOSS value: the game works without depending on anyone's server.

6. **Solo play runs a local arbiter.** Whether this arbiter is "secretly multiplayer" (registerable on the hub for
   others to join) is a UX toggle, not an architectural change.

---

## Open Questions (Still Unresolved)

### 1. Does the dedicated server handle lobbies, or only missions?

The author's instinct says the dedicated server (`unhaunter_dedicated`) should "jump straight to mission" and not handle
the lobby. But the lobby is envisioned as an in-game Bevy space, which requires Bevy to simulate. If the lobby doesn't
run on the dedicated server, where does it run?

Options not yet chosen:

- Lobby and mission on the same server process (current implementation). Simple, but idle lobbies cost CPU.
- Hub manages lightweight lobby state (just player list + chat), server only spins up for missions. Breaks the
  in-game-lobby-as-physical-space vision.
- Lobby is client-hosted, mission is on dedicated server. Player handoff complexity.

### 2. Multi-room per process: feasible?

If a single Bevy headless process could run multiple independent game worlds (using `SubApp`, `World` isolation, or
similar), the per-room scheduler overhead drops dramatically. This would make the dedicated server model unambiguously
the right choice. But it's unexplored territory — the author has "NOT analyzed this if it's even viable."

### 3. When does the hub get built?

The author says "very very soon." It's the missing piece for the shippable experience. But its scope is unclear — does
v1 of the hub just do room codes? Or does it also need a server browser, ban lists, presence?

### 4. Voice chat timeline

Voice is critical for public rooms with strangers. Without it, multiplayer with strangers is fundamentally broken — this
is a game about _talking about evidence_. But voice is a massive infrastructure and UX lift. The tension: the author
wants public rooms soon, but public rooms without voice are hollow.

### 5. What's the actual idle CPU of the optimized server?

The 4× improvement from single-threading is an estimate. The adaptive tick rate benefit is theoretical. These need to be
measured. The 240-room projection depends on these numbers being real.

---

## The Emotional State of the Project

This section exists because the author asked that the document capture "what is bothering me" and "the journey."

The author is excited about the multiplayer progress. Plans 21-27 delivered a functional multiplayer game from nothing.
The lobby works. Late join works. Disconnect protection works. The dedicated server runs headless. This is real,
tangible, demonstrable progress.

But the excitement is competed by anxiety about direction. There's a feeling of being at a crossroads where committing
to any path means closing others, and the author isn't confident enough in any single path to close doors. The relay
model is proven by Phasmophobia but creates the "who hosts?" problem. The dedicated server model solves that but has
scaling concerns. Supporting both is tempting but doubles the maintenance burden.

The strongest emotional signal in the conversation: **the need to ship.** The author wants to get multiplayer into
players' hands, wants something "worth bragging about," wants to put pressure on the timeline. The decision paralysis is
felt as actively blocking that goal. Every hour spent deliberating is an hour not spent building.

The second strongest signal: **FOSS values are non-negotiable.** The game must work without central infrastructure
(hence `--host`/`--join` stays). It must not require accounts (hence UUID/keypair identity). Infrastructure costs must
be sustainable by a solo developer without external funding. These aren't preferences; they're constraints.

The third signal: **community anxiety.** Will anyone play? Will the effort be wasted on infrastructure nobody uses? The
5-player problem haunts every architectural decision. The author would rather build community tools (bulletin boards,
scheduled events, the "mixer" mode) than perfect server infrastructure, because the limiting factor isn't technology —
it's people.

---

## Comparison with Earlier Document (2026-02-09)

### What changed

| Aspect                  | 2026-02-09                       | 2026-02-14                                               |
| ----------------------- | -------------------------------- | -------------------------------------------------------- |
| Dedicated server stance | "The clear next step, build it"  | "I have it, but I'm questioning whether to keep it"      |
| Relay stance            | "Rejected — abuse problems"      | "Back on the table, Phasmophobia does this"              |
| Hub/discovery urgency   | "Phase 4, eventually"            | "Need this very very soon"                               |
| Server CPU concern      | "Needs profiling, uncertain"     | "Measured: ~10% idle, but optimizable to maybe 2.5%"     |
| Scalability estimate    | "Unknown"                        | "~240 rooms on €10/month VPS seems like enough headroom" |
| Confidence level        | Decisive (dedicated server wins) | Uncertain (revisiting the decision, decision paralysis)  |

### What stayed the same

- `--host`/`--join` stays as off-grid mode.
- Identity = UUID → keypair (no accounts).
- Lobby should be a physical in-game space.
- Voice chat is critical but deferred.
- The game is about attention, not reflexes. Multiplayer is about team coordination, not competition.
- The 5-player problem is the real challenge, not technology.
- Community servers (Minecraft model) are the long-term scalability answer.

---

## Summary of This Conversation's Key Insights

1. **Relay and dedicated server are wildly different products**, not variations of the same idea. A Tokio relay handles
   thousands of rooms but understands nothing. A Bevy headless server understands everything but runs one room per
   process. The choice between them shapes the entire architecture.

2. **The capacity math might resolve the decision.** If dedicated server optimization (single-threaded, lower tick rate)
   achieves ~240 rooms on a €10/month VPS, the relay's scaling advantage becomes irrelevant for a FOSS game's realistic
   player counts.

3. **The hub is the highest-priority unbuilt piece.** It's what makes multiplayer _feel_ like a product rather than a
   developer tool. Room codes instead of IPs. A server browser. The social layer.

4. **Host mode and dedicated server share the same arbiter code.** The "extra code paths" concern is mitigated if the
   arbiter is a clean, separable piece. Host mode = arbiter embedded in client. Dedicated mode = arbiter standalone.
   Same code, different deployment.

5. **The lobby boundary question is the ugliest open problem.** Where does the lobby simulation run? This question
   connects the dedicated-server-vs-relay decision, the in-game-lobby vision, the solo-as-multiplayer goal, and the cost
   concern. It hasn't been answered.

6. **The author's biggest fear isn't technical — it's maintenance burden.** Supporting relay + dedicated server + host
   mode + hub + community servers as a solo developer is daunting. The question isn't "can I build this?" but "can I
   sustain this?"

7. **Ship something, then iterate.** The decision paralysis is more damaging than picking the wrong option. Any of the
   viable paths (dedicated server with hub, or relay with hub) can work. The risk of not shipping exceeds the risk of
   picking the suboptimal architecture.
