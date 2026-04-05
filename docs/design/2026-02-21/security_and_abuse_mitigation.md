# Security & Abuse Mitigation: The Friction Architecture

**Date:** February 21, 2026

**Authors:** David Martínez Martí & Gemini 3.1 Pro

> _"If you can saturate a server with a REST call, it will happen 100% guaranteed. But you don't need to build
> Cloudflare in Rust; you just need enough friction to make the attack exhausting."_

## 1. Introduction: The Asymmetry of FOSS Multiplayer

In a traditional AAA game, banning a troll works because they lose a $30 license. In a Free and Open Source (FOSS) game,
the economic asymmetry is brutal. Banning an IP address is useless against VPNs, and banning a free account takes ten
seconds to bypass. Worse, an attacker with a 3-line Python script can theoretically consume 100% of a server's RAM slots
in milliseconds, holding them hostage and ruining the Friday night of 50 legitimate players.

Attempting to build an impenetrable, enterprise-grade anti-DDoS system for a ghost-hunting game is a trap that leads to
infinite maintenance burden. Volumetric network attacks (L3/L4) are the ISP's problem.

Our job is to protect the application layer. This document outlines the **Friction Architecture**: a layered defense
that protects the infrastructure mathematically and socially. We cannot charge trolls money, so we must charge them
their most valuable resource: **Time**.

## 2. Identity & "Just-In-Time" Consent

To tax a troll's time, the server must be able to recognize them. However, traditional accounts (emails/passwords) add
massive friction to onboarding and introduce severe GDPR liabilities.

**The Silent Public Key:** Instead of an account, players are identified by a local cryptographic Public/Private Keypair
generated directly on their machine. Because a Public Key is just a mathematical string with no attached Personal
Identifiable Information (PII), the GDPR footprint remains incredibly light.

**Just-In-Time Consent:** The game remains completely anonymous and offline by default. The moment the player clicks
"Multiplayer" for the very first time, they receive a transparent popup: _"To play online, Unhaunter needs to generate a
local Public Key to manage your multiplayer reputation. This key is stored locally, but because it identifies your
installation to our servers, it could be used to track your gameplay history. [Accept] or [Return to Single Player]."_

This fulfills the GDPR "Gold Standard" of explicit consent. If they later wipe their local multiplayer data, the key is
deleted, instantly fulfilling their "Right to Erasure"—and forcing them to start their reputation grind over from
scratch.

## 3. The Reputation Economy & The Mixer (Pest Control)

The core of our anti-troll philosophy is simple: **You must prove you are a good actor before you are given the power to
annoy people.**

When a new Public Key is generated, it starts at **0 Reputation**. A 0-Reputation player cannot host a Public Lobby on
the server browser. To earn the right to host or join high-stakes public games, they must spend time playing the game
properly.

### The Mixer as "Proof of Work"

To earn Reputation, new keys must queue into **The Mixer** (Streamer Chaos Mode / Quick Play). The Mixer is a
server-controlled, fast-paced matchmaking queue where players do not choose the map, difficulty, or their teammates.

- _Why it defeats trolls:_ A troll cannot farm Reputation by hosting a private lobby and AFKing with three of their own
  alt accounts. The server controls the Mixer. The troll is forced to play with strangers. If they grief, the strangers
  vote-kick them, and they earn zero Reputation. It makes farming mathematically exhausting.

### Defeating the "Gatekeeper Troll" (The Hell Lobby)

What happens if a dedicated troll decides to camp the Mixer, intentionally sabotaging missions to ensure genuine newbies
never earn enough Reputation to escape?

We do not explicitly ban them. An explicit ban simply tells a troll to rotate their VPN and generate a new key. Instead,
we use **Shadowbanning**. Flagged Public Keys are silently routed to a specific ProcMan configured for "Hell Mode." The
server tick rate drops to 5Hz, physics are crippled, and the lobby is populated _only_ with other flagged trolls. They
think they are ruining the game for newbies, but the server has actually quarantined them in a zero-resource digital
sandbox. We waste hours of their time for pennies of CPU cost before they realize they need to start over.

### The Vouching System

To prevent the Mixer from feeling like a grind for genuine friends who just bought the game, we allow the "Web of
Trust." A veteran player (Rep 10) can "Vouch" for a newbie (Rep 0), instantly granting them Rep 1 so they can skip the
Mixer.

- _The Trap:_ The vouch is a cryptographic chain. If the newbie trolls and gets banned, the veteran's Public Key is
  blacklisted too. Trolls cannot trick veterans into vouching for burner accounts.

## 4. Defeating the REST Loop (The Network Defense)

If a server allocation is triggered by an unauthenticated REST API call (`POST /v1/rooms/create`), the attacker has a
3000:1 advantage. One HTTP request (100ms) allocates a 100MB Bevy RAM slot for 5 minutes.

We flip this asymmetry by moving the "cost of entry" away from the stateless REST API.

- **Stateful TCP Handshakes:** The REST call does not spawn a heavy dedicated server. It only creates a lightweight,
  in-memory reservation. To actually claim the server, the client must connect to the ProcMan and complete a **stateful
  TCP handshake**. The attack requirement shifts from a 3-line Python `curl` script to a custom, distributed, stateful
  networking application.
- **10-Second Fast Expiry:** If a room is reserved via REST but the TCP connection is not established and validated
  within 10 seconds, the room is vaporized.
- **Per-IP Room Cap (Max 2):** An in-memory map tracks how many active rooms an IP holds. A 3rd request returns a
  `429 Too Many Requests`. Even with a sophisticated script, an attacker can only hold 2 servers hostage, and they are
  evicted 10 seconds later.

## 5. Infrastructure Hardening (Operational Reality)

A custom protocol deters casual script kiddies, but basic application hardening prevents catastrophic failures. During
our internal IDE audit of the V1 Hub, we identified massive vulnerabilities that must be permanently ruled out:

- **The Unbounded Codec Trap:** Using tools like `LinesCodec::new()` without a max length limit is a death sentence. A
  single attacker with `netcat` can connect to an exposed port, send an infinite string of garbage without a newline
  (`\n`), and cause an Out-Of-Memory (OOM) crash in seconds. All codecs must be strictly bounded
  (`LinesCodec::new_with_max_length`).
- **Port Exposure:** Internal control ports (like the ProcMan port 11000) must never be bound to `0.0.0.0` without
  strict firewalling, mTLS, or VPN tunnels. The Hub must be the only public-facing ingress point.

## 6. Dynamic QoS (DDoS Lifeboats)

Even with IP caps, a distributed botnet could theoretically exhaust the server pool. We defend against this by
dynamically shifting the cost of hosting based on server load.

- **Normal Load:** Private room hosting costs **0 Reputation**. Two brand new friends can instantly play together
  privately.
- **High Load (Under Attack):** If the VPS detects 80%+ capacity, the Hub flips the QoS flag. Hosting a Private Room now
  requires a Cryptographic Reputation Score of `>= 1`. Bots with freshly generated keys are instantly dropped in RAM.
- **The "Party Sum" Exception:** Reputation is aggregated across the connecting lobby. If one veteran (5 Rep) joins
  three newbies (0 Rep) during a DDoS, the lobby's total sum passes the check. The established community naturally
  shields and carries new players through the firewall during an attack.

## 7. Moderation: The "Doom Demo" Architecture

Voice chat (SFU routing) is essential for Unhaunter, but moderating it presents a massive GDPR and storage liability.
Recording all voice chat on the server is surveillance, and it is legally indefensible.

**The Solution: The 60-Second Client Buffer.** The game client maintains a rolling 60-second buffer of both
incoming/outgoing audio _and_ 3D/2D spatial telemetry inputs entirely in local RAM.

If everyone is playing nicely, the buffer constantly overwrites itself. It never touches a hard drive. If a player
actively griefs or screams a slur, the victim clicks "Report Player." _Only then_ does the client package that highly
compressed, 60-second buffer, encrypt it, and upload it to the Hub as a ticket.

Moderators do not just get a "he-said-she-said" audio clip. They receive a literal `.lmp` style "Doom Demo" replay of
the exact spatial interaction and audio that occurred. The troll's behavior is captured with zero ambiguity, and the
server retains zero unnecessary surveillance data.
