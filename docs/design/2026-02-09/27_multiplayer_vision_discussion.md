# Multiplayer Vision Discussion — 2026-02-09

A free-form conversation about the future of Unhaunter's multiplayer: high-level planning, UX vision, social design, and
networking infrastructure. No code was written. This is a record of ideas explored, not decisions made.

---

## Starting Point: Where Multiplayer Is Today

The game was single-player only until ~2 weeks ago. A network multiplayer approach was retrofitted that works for a 1:1
host + client approach:

- Host starts with flags `--host 5000 --map <map> --difficulty <difficulty>`
- Client starts with `--join IP:PORT`
- Protocol: TCP + JSONL
- Host-authoritative for most things, with some parts being reverted to client-local (movement, hiding)
- 2-player only

While it works, this approach is very unusable for end users.

### Current Technical State

- **Transport:** Raw TCP sockets, JSONL serialization
- **Authority:** Host-authoritative for majority of things. Client does movement and hiding locally.
- **Entity identity:** `NetworkId(u64)` based on hashing tile properties for map entities
- **Snapshot system:** A 22-field `SnapshotMsg` struct broadcast from host to client at ~60Hz
- **Known architectural issues:** God file (~2300-line systems.rs), state triplication, client running game logic it
  shouldn't, no sync abstractions, dual NetworkId types

---

## Long-Term Goals (Discussed)

- **Player count:** Up to 9 players per mission
- **Platforms:** Desktop-first, WASM later (transport difference only)
- **Hosting:** Players can host games, dedicated servers should be possible, eventually some central infrastructure (but
  cost-conscious — FOSS game, solo dev)
- **Identity:** No accounts initially (pick a name, join). Keypair-based identity considered for banning/trust.
  Agency-signed certificates as a concept for gatekeeping without central authority.
- **P2P consideration:** Acknowledged as interesting for reducing server costs, but star topology (1 authority, N-1
  clients) is the right call for up to 9 players. Full mesh (36 connections) is messy. Host migration is "maybe someday"
  territory.

---

## Three Separate Concerns

The discussion identified three independent layers that can evolve at different speeds:

1. **Topology** — who connects to whom, who computes what
2. **Session flow** — how players find each other, pick a mission, start playing
3. **Transport** — how bytes move (TCP, WebSocket, WebRTC)

---

## Phased Roadmap (High-Level)

### Phase 1: Multi-Client (N players on 1 host)

Everything depends on this. Host needs to accept more than 1 client. `NetworkConn` becomes a collection. Snapshots
broadcast to all. Input arrives from N sources.

Key design decision: make "host player" just another player entity (not special-cased) to set up for dedicated servers
later.

**Milestone:** 3-4 players in a mission, still CLI-launched.

### Phase 2: In-Game Session Setup (Lobby)

Replace CLI flags with an in-game flow:

1. Player opens game normally (main menu)
2. Chooses "Host Game" or "Join Game"
3. Host picks port, sees "waiting for players" screen, picks map + difficulty
4. Client enters IP:port (or room code), sees lobby, sees other players
5. Host hits "Start Mission"

Protocol change: Split the `Welcome` message into two phases:

- **Pre-mission phase:** `LobbyState { players, map, difficulty }` keeps everyone in sync
- **Mission start:** `StartMission` message (today's `Welcome` payload)

**Milestone:** Players host/join via game menu, see each other, start together.

### Phase 3: Dedicated Server

A host that doesn't render and doesn't have a local player. If Phase 1 is designed right (host player isn't
special-cased), this is almost free:

- Second binary or `--dedicated` flag
- Skips rendering/audio/input plugins
- Reads config from file or CLI
- Bevy supports headless mode (`MinimalPlugins`)

A cheap VPS could run many concurrent instances — the simulation isn't heavy for a tile-based 2D game.

**Milestone:** A headless binary that can be run on a VPS.

### Phase 4: Discovery (Server Browser / Coordination)

The central service is trivially cheap — it's not running game logic:

- Servers register themselves (heartbeat with IP, port, player count, map, status)
- Clients query the list (GET endpoint returns JSON)

Options from simplest to most complex:

1. Static list / Discord bot (zero cost)
2. Simple REST service (~200 lines, in-memory store, stale entries expire)
3. Matchmaking service (overkill for current player base)

### Phase 5: Transport Abstraction

When WASM becomes real. Abstract behind a trait:

- TCP implementation for desktop
- WebSocket/WebRTC for WASM
- Game logic doesn't care

---

## UX/Design Vision: The Lobby

### Phasmophobia's Physical Lobby — Why It Works

- **Shared presence before commitment.** See each other, goof around, feel like a group.
- **Implicit readiness.** Walk to the van → mission starts. No "ready up" button.
- **Configuration as interaction.** Whiteboard, equipment shelf — diegetic UI.
- **Free onboarding.** Players learn movement and interaction before encountering a ghost.

For Unhaunter this maps naturally: the game already has tile-based environments, character movement, object interaction.
A lobby room is just another map, reusing the entire engine.

**Key insight: the lobby IS the tutorial.**

### The "Always Connected" / Persistent Home Idea

Concept: You load the game and you're in YOUR lobby — your house/apartment. Interactive space instead of a menu. Solo
play, options, etc. are accessed through in-world interaction.

Social extensions imagined:

- Visit friends' houses
- Meet at someone's place to do missions together
- Social gathering servers (community-hosted maps, no ghost, proximity chat)

#### Tensions for a Solo Dev FOSS Project

1. **Server costs scale with online time, not play time.** Persistent worlds cost more than session-based.
2. **Minimum viable social experience has a high bar.** A persistent house alone feels lonely in a bad way if there
   aren't enough concurrent players.
3. **Scope creep pressure.** Customization, furniture, visiting, messaging — individually small, collectively massive.

#### The Local-First Middle Ground

- Launch game → you're in your house (offline, local save, like a fancy main menu)
- Walk to in-game computer/crystal ball → shows friends online, open rooms (diegetic server browser)
- Walk to front door → "Host a game" (friends can now visit)
- Friend visits → both walk to mission board → pick a job → step into the van
- After mission → back in host's house → debrief → friend leaves → goes to own house

This gives persistent-home _feeling_ without persistent-server _cost._ Degrades gracefully: even with zero friends, you
have a cozy space to manage your profile and start solo missions.

### Social Hubs as Emergent, Not Engineered

Concept A (Personal Lobby): Private space. Your place. Friends visit. Mission staging. Concept B (Social Hub): Public
space. Community-hosted. Strangers hang out. Meeting people.

These have different requirements:

|                      | Personal Lobby         | Social Hub                    |
| -------------------- | ---------------------- | ----------------------------- |
| Hosting              | Player's machine (P2P) | Community server or dedicated |
| Persistence          | Per-session            | Semi-permanent                |
| Player count         | 2-9 (your party)       | Potentially dozens            |
| Central infra needed | Presence/friends list  | Server directory              |

**The social hub doesn't require new tech.** It's a multiplayer session on a map with no ghost. A community member could
run one the moment multi-client + dedicated server support exists. The approach: build the infrastructure that makes it
possible, let it emerge.

Required pieces:

1. Multi-client support
2. Dedicated server binary
3. Server browser / directory
4. A "no mission" mode (server config flag)
5. Proximity voice/text chat

---

## Distilling the "Why" Behind Always-Connected

The underlying needs, separated from specific solutions:

### Need 1: Zero-Friction Matchmaking with Strangers

Multiplayer today requires pre-existing relationships. Most players who download the game won't have a friend who also
plays. A solo player should be able to launch and end up playing with someone without planning it.

### Need 2: Eliminate the "Decision to Play Multiplayer"

Blur the single/multi boundary. Multiplayer can just... happen. The loneliest moment in multiplayer gaming is deciding
to look for a game and finding no one. If the player never explicitly searches, they never experience that rejection.

### Need 3: Presence Without Intent

People should be _around_ even when they haven't decided to play a mission. Existing, visible. This creates conditions
for spontaneous interaction.

### Need 4: Lower the Social Barrier

Hosting is a power move. Joining a stranger is socially vulnerable. Meeting should feel equal, casual, like bumping into
someone in a shared space.

### Need 5: Health Markers for the Developer

An always-connected approach (even just presence) provides a legitimate mechanism to capture player counts and retention
stats. For a small FOSS project, seeing "5 people are playing right now" is a vital heartbeat that anonymous download
logs can't provide.

### The Core Question

Can this game feel like a **place** rather than a **program**?

A program: launch it, do a thing, close it. A place: go there, things happen, leave when done.

"Place" keeps people lingering, lingering creates overlap, overlap creates spontaneous co-op.

---

## The Soul of the Game

### Unhaunter's Core Identity

> "Unhaunter wants to be a game where the player's skill is _paying attention._"

Not reflexes. Not memorizing patterns. Not executing mechanics. _Attention._ Noticing a 0.3 degree temperature drop.
Hearing static texture changes on the spirit box. Seeing the EMF reader twitch directionally. The game rewards the
player who listens to the house.

The motto: **No arcade stuff, no random stuff, no scripted stuff. Simulate and let players discover.**

### Specific Inspirations & Aesthetics

- **"El Orfanato" (The Orphanage):** The film's depiction of a technical haunting is a core reference. The goal is to
  shift from powerless fear to domination through surveillance — deploying massive amounts of gear, remote monitoring,
  and watching sensors trigger in real-time.
- **Liminality over Grime:** A specific rejection of the "grunge/clutter" aesthetic common in horror games. Maps should
  feel "softer," flatter, and more liminal (like early Phasmophobia) rather than over-detailed or abandoned sets. The
  horror comes from the subtle corruption of a normal, clean space.

### What Multiplayer Means for This Game

Initially framed as "shared interpretation of ambiguous evidence" — players discussing readings, disagreeing, convincing
each other. But the creator's vision is more specific:

**Team building. Becoming the SWAT team of ghosts.**

The arc: from powerless chaos (nobody knows what to do, everyone in the same room) to dominating the haunting through
knowledge, good practices, and team coordination.

The game doesn't teach procedures — it gives systems deep enough that procedures _emerge._ Players build their own
playbook.

Multiplayer cooperation should involve:

- Lots of deployable gear (cameras, sensors) monitored from the van
- A van that's a real workplace, not a safe room
- Natural role differentiation through equipment and positioning, not enforced roles
- Quick coordinated deployment ("understanding what needs to happen")

### The Van as Operations Center

The van shouldn't be where you hide — it should be where _work happens._ Screens, camera feeds, sensor readouts, house
map with data overlays. The van operator is the most information-rich player, directing the field team.

Natural 2-role split at minimum player count:

- **Field team:** Inside, deploying equipment, taking readings, encountering the ghost
- **Van operator:** Outside, watching the big picture, correlating data, warning field team

Scaling happens naturally (1-2 van, 3-4 field at 5 players; managing multiple screens at 9).

**The van operator role is perfect for new player onboarding.** Safe, learning, seeing what readings look like,
contributing (calling out data) without needing to know procedures yet.

---

## The 5-Player Problem

With ~5 players worldwide across different timezones, synchronous matchmaking is impossible. Lobbies, server browsers,
social hubs — all useless at that scale.

### Asynchronous Presence

The game can't rely on "be online at the same time." It needs to help players find each other _across time._

**A bulletin board, not a lobby.** When you launch, you see messages from other players. Posted hours or days ago.
"Looking for someone for the asylum, I play evenings UTC+1." Persists on a lightweight central service. Works at any
player count, even 5.

**Scheduled sessions.** Post in-game: "I'll host Saturday 8pm UTC." Others see it and mark interest. This is what
players already do via Discord — pull that coordination into the game.

**"I was here" traces.** Signs of other players' existence even without real-time multiplayer. A news ticker: "Agency
'Night Owls' completed Riverside Manor (3 players, 47 min)." You're alone, but you know others exist.

### Contacts, Not Guilds

The "agency" shouldn't be a rigid guild structure. It's a **contact list with history.**

- Investigate with someone → add as contact
- Contacts show online/offline/last seen
- See who you've played with, when, how many times
- "Investigated with Alex 12 times" tells a new player "this is a regular"

An agency is just a label a group of contacts gives themselves. Shared name and icon. No mechanics, no permissions, no
hierarchy. Fluid — you can be in multiple agencies or none.

This avoids guild politics while giving groups identity.

### Priority

The medium-term social features (bulletin board, contacts, play history) matter more than lobbies or dedicated servers.
The 5-player problem is solved by async social tools, not better infrastructure.

---

## NAT/Networking Problem

### The Specific Situation

- Creator: behind CGNAT, can't port-forward IPv4. Has IPv6 but friend's ISP doesn't support it.
- Friend: can NAT IPv4, can host. Forces him to always be host.
- SSH tunnel to VPS: works but adds significant latency.
- Desired: any player can host, or nobody has to think about it.

### Option 1: TCP Relay Server (Recommended Near-Term)

A tiny relay service on the VPS. Both players connect outbound (always works through NAT/CGNAT). Relay forwards packets
between them. Dumb pipe — doesn't understand the game protocol.

- Works for everyone, always, regardless of NAT
- Tiny service, ~100 lines of code
- Works with current TCP protocol
- Near-zero CPU and ~10-50 KB/s bandwidth
- Added latency: probably 5-20ms if VPS is geographically reasonable (imperceptible for a non-twitch game about reading
  thermometers)
- Room concept: host creates room (gets code), client joins with code, relay pairs them

### Option 2: UDP + NAT Hole Punching (STUN-style)

Coordination server helps clients discover each other's public IP:port. Simultaneous UDP packets create NAT pinholes.
Direct connection.

- Direct connection once established — minimal latency
- Server costs near zero (handshake only)
- **But:** CGNAT success rate is 50-80%, depending on implementation. Some CGNATs use random port mapping, making
  prediction impossible.
- Requires TCP→UDP migration (need reliability layer: laminar, quinn/QUIC, enet)
- **Every real implementation includes a relay fallback.** You build Option 1 anyway.

### Option 3: Relay-First, Hole-Punch-Upgrade (Best Architecture)

1. Both connect to relay (always works)
2. Background: attempt UDP hole punching
3. If hole punch succeeds, seamlessly migrate to direct connection
4. If fails, keep relay — player never notices

This is what Steam Networking, Discord voice, and most modern games do. Most work, but nothing gets thrown away.

### Option 4: Hosted Game Server on unhaunter.com

Run headless Unhaunter on the VPS. Zero NAT problems.

- Advances dedicated server goal
- But couples "NAT solution" with "dedicated server" — different problems
- VPS runs actual game simulation (CPU usage, though minimal for 2D tile game)
- A $5/month VPS could handle 10+ concurrent games

### Recommended Path

1. **Now:** Simple TCP relay on VPS. Weekend project. Solves NAT immediately.
2. **Later (UDP migration):** Add hole punching with relay fallback. Relay evolves from TCP forwarder to STUN+TURN.
3. **Later (dedicated servers):** They connect to same relay infra or listen directly on VPS.

Each step builds on the last. Nothing thrown away.

### Cost Reality

- Relay: negligible. $5/month VPS handles dozens of concurrent games.
- Dedicated game server: modest. 2D tile-based simulation is ~2-5% of a core per instance. $5 VPS handles 10+ concurrent
  games.
- Cost only matters if you get popular — the best problem to have.

---

## Summary of Key Insights

1. **The game's multiplayer identity is team building through attention** — becoming competent together, developing
   shared procedures, the SWAT fantasy.

2. **Voice chat is the game.** Not a nice-to-have. The core multiplayer mechanic is real-time conversation about
   ambiguous evidence and coordinated response.

3. **The social problem at small player counts is async, not sync.** Bulletin boards and scheduled sessions matter more
   than lobbies and server browsers.

4. **The lobby should be a physical space** — another map, reusing the engine, serving as both social space and
   tutorial.

5. **The van is the onboarding path** — new players learn by operating from safety, graduate to the field as they
   understand systems.

6. **Agencies are fluid contact groups with history**, not rigid guilds.

7. **A TCP relay solves the NAT problem immediately** for near-zero cost. UDP + hole punching can come later as an
   upgrade, with the relay as fallback.

8. **Build social connective tissue before infrastructure.** The 5-player problem is solved by human connection tools,
   not better servers.
