# Hub Architecture v2: The "Zero-Ops" Network

**Date:** February 21, 2026

**Authors:** David Martínez Martí & Gemini 3.1 Pro

> _"State is the enemy of the sysadmin. If a server goes down, the recovery plan shouldn't be a three-day database
> migration; it should be a 45-second script."_

## 1. Introduction: The FOSS Multiplayer Dilemma

Building multiplayer for a Free and Open Source (FOSS) game presents a paradox. If we rely on pure Peer-to-Peer (P2P),
players are blocked by CGNAT and port-forwarding nightmares. If we build a centralized server backend, the solo
developer is suddenly burdened with monthly AWS bills, database maintenance, and GDPR compliance.

Our time is valuable. Every hour spent doing manual server administration is an hour not spent making the game fun.
Therefore, Unhaunter’s networking architecture is built around a "Zero-Ops" philosophy. We want the frictionless UX of a
centralized service (like Steam), but the operational and legal resilience of a federated network (like Mastodon).

### 2. The Universe, The Domain, & The Stateless Hub

To achieve frictionless matchmaking without centralizing liability, we divide the network into distinct trust
boundaries.

- **The Universe (The Directory):** A Universe defines an isolated ecosystem of servers. While the "Official" Universe
  is managed by the core team via a public GitHub repository (`universe.json`), the architecture explicitly supports
  **multiple, independent Universes**. A Universe acts as a static directory of server operators trusted by _that
  specific Universe's team_. If a community wishes to completely fork the ecosystem, they can host their own registry.
  Players can point their game client to a new Universe via a simple config file tweak (kept out of the UI for v1).
  Currently, Universes are strictly isolated from one another with no inter-Universe discovery.
- **The Domain (The Operator):** A cohesive infrastructure unit (Hub + ProcMans + Dedicated Servers) operated by a
  trusted entity within a Universe. We identify Domains using a simple "Area Code" prefix on the room code.
  - `U-XXXX` routes to the Official Unhaunter domain.
  - `E-XXXX` routes to a community-run "Example" domain.

This Area Code system is our primary UI and legal shield. When a user types `E-XXXX`, the client instantly knows it is
connecting to a community server. It fetches the operator's `domain.json` (containing their specific Privacy Policy) and
displays a "Community Server" badge. This fulfills "Informed Consent" without annoying legal popups, shifting data
controller liability to the independent operator.

**The Stateless Hub:** Crucially, the Hub software itself is strictly **RAM-only**. It routes connections and holds
active room states in memory, but it writes no player session histories or IP logs to a persistent database. If the Hub
crashes or restarts, the routing table is wiped clean. By ensuring the Hub is a "Transient Processor," we achieve the
lowest possible GDPR risk profile and eliminate database maintenance entirely.

## 3. Codebase Drift & The WASM Reality

Historically, the game supported multiple execution paths: Single Player, `--host` (where a player's machine runs the
simulation), and connecting to an `unhaunter_dedicated` headless server.

Maintaining `--host` alongside `unhaunter_dedicated` creates a massive architectural trap: **Codebase Drift**. When bugs
appear in the dedicated server path but not the `--host` path, the developer is effectively maintaining two different
games.

**The Unification Directive:** Architecturally, the `--host` mode must execute the exact same logic as the dedicated
server. In practice, choosing to "Host" locally should simply spin up a hidden background dedicated process and
auto-connect the client to `localhost`.

**The WASM Constraint:** WebAssembly (WASM) does not cleanly support the heavy multithreading required to run the Bevy
server scheduler in the background. Instead of contorting the architecture to make this work, we are establishing a hard
rule: **WASM is strictly Client-Only.** Players playing via a web browser can join any room (Public, Private, or Mixer),
but they cannot host a local server. If they want to host off-grid, they must download the native desktop client.

## 4. Zero-Ops Deployment (Curing Migration Dread)

The true test of an architecture is what happens when the underlying VPS provider fails, or a server needs to be
migrated. In legacy web administration, this meant days of moving SQL databases and tweaking socket configurations.

Unhaunter's deployment must be entirely stateless and executable in under a minute.

- **Ansible Playbooks:** We reject "cowboy" `curl | bash` scripts. To build trust with community operators, deployment
  is handled via declarative Ansible playbooks. An operator reads the playbook, sees exactly what ports are opened and
  what binaries are fetched, and runs it safely. It is self-documenting and idempotent.
- **Caddy Integration:** By using Caddy as our reverse proxy, we automate Let's Encrypt SSL/TLS certificate
  provisioning. Complex HTTPS configuration is reduced to a few lines of text.

Because the Hub uses local `.ron` files for static configuration and RAM for active state, migrating a server simply
means pointing the DNS record to a new Ubuntu box and running the Ansible playbook. Total downtime: 45 seconds.

## 5. Voice Chat Routing: The SFU Imperative

Proximity voice chat—hearing a teammate's voice muffle through walls or cut to static when the entity hunts—is not a
side feature; it is the core gameplay loop.

Initially, we considered WebRTC Peer-to-Peer (P2P) to route audio directly between clients, saving server bandwidth.
However, P2P in a public matchmaking setting (The Mixer) is a catastrophic security and privacy trap. P2P requires
exposing every player's raw IP address to every other player in the lobby via STUN, enabling malicious actors to easily
launch DDoS attacks or swatting attempts. Furthermore, broadcasting IPs unnecessarily violates the GDPR Data
Minimization principle.

**The Solution: Selective Forwarding Unit (SFU)** We must route all voice traffic through the Hub/Dedicated Server. The
client sends one audio stream to the server, and the server relays it to the other players.

- **Privacy & Security:** The server acts as a proxy, completely masking player IPs from each other and bypassing the
  need for complex NAT hole-punching (STUN/TURN) on the client side.
- **Bandwidth Reality:** Modern Opus codecs use roughly ~64kbps per player. A standard €10 VPS with a 1 Gbps network
  link can route hundreds of concurrent rooms without breaking a sweat.
- **The Trade-off:** We willingly accept a slight increase in voice latency (ping/RTT) in exchange for absolute legal
  compliance, player safety, and network simplicity.
