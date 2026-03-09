# 19 — Client Authority and Replicon Visibility Synthesis

- **Date:** March 2026
- **Branch:** `dev-deavid`
- **Purpose:** A distilled overview of how we will solve client ownership using `bevy_replicon` 0.39, synthesizing the
  research from the previous document into a concrete understanding of our tools and the chosen path.

---

## 1. The Core Issue: Fighting the Server

Currently, the owning client experiences jittery, laggy movement when controlling their local player.

**Why it happens:** When a client takes ownership of an entity, it updates the `Position` locally based on user input
and sends those updates to the server. The server happily accepts the new position and replicates it to everyone else.
The problem? The server also replicates that position _back to the owning client_. Even though the client removed the
`Replicated` component locally, the server still sees the entity as replicated. Consequently, the client's local
position is continually overwritten by the server's older, replicated state.

The desired architecture requires strict authority boundaries:

- **Owning Client:** Authoritative over its local player and held gear. Updates position locally, broadcasts to server.
- **Server:** Authoritative over ghosts, interactive objects, and untouched gear.
- **Other Clients:** Strictly receive replicated views of everything they don't own.

## 2. Available Tools in `bevy_replicon` 0.39

To stop the server from overwriting client-owned data, we looked at two primary APIs in version 0.39:

### Server-Side: Visibility Filters

These dictate what the server sends over the network.

- Implemented via the `VisibilityFilter` trait. Both the filter and `ClientComponent` must be strictly immutable.
- **Entity Scope:** Hides the whole entity.
- **Component Scope:** Hides specific components (e.g., `Position`).
- **The Catch:** If an entity or component loses visibility _after_ it has already been replicated (e.g., picking up a
  piece of gear), the server sends an explicit **despawn** or **component removal** message.

### Client-Side: Receive Markers

These dictate how the client processes incoming data.

- The client can register custom functions (`WriteFn` and `RemoveFn`) to handle updates for entities bearing a specific
  marker (like `LocallyOwned`).
- We can implement a `noop_write` to silently discard incoming server components, and a `noop_remove` to ignore server
  deletions.
- **Critical Requirement:** A custom `noop_write` _must_ consume the associated message bytes using
  `rule_fns.consume()`. If it doesn't, the byte stream desyncs and corrupts all subsequent updates in the packet.

### The Design Mismatch

`bevy_replicon` is fundamentally built around **server authority**. There is no built-in `ClientAuthority` concept.
Visibility filters are meant for "fog of war" mechanics, and receive markers are meant for client-side prediction
reconciliation. Repurposing them to force a strict client-authority model carries inherent edge-case risks.

## 3. Evaluating the Strategies

We evaluated multiple approaches to prevent the server's overwrites. Our findings:

- **Entity Blacklisting Post-Spawn (Failed):** Banning an entity on the server after it was replicated sends a despawn
  message, destroying the entity on the client.
- **Component Blacklisting Post-Spawn (Failed):** Hiding just `Position` sends a removal message. Even if the client
  quickly re-adds the component, we suffer a 1-frame visual flicker.
- **Filtering at Spawn Time (Incomplete):** Hiding components at the exact moment of spawning works perfectly for player
  entities (which are "owned" from the start), but fails for gear picked up mid-session, triggering the removal flicker
  again.
- **Receive Markers (The Chosen Solution):** By adding custom no-op receive markers to `LocallyOwned`, the client simply
  ignores the server's replication overwrites. Since the server's messages are safely discarded (with bytes properly
  consumed), there are no despawns or removal flickers, even when gear is picked up mid-game.

## 4. Conclusion and Next Steps

The most robust, flexible solution is **Strategy E/F**: relying primarily on **Client-Side Receive Markers** bound to
the `LocallyOwned` component.

By having the client ignore both write and remove commands from the server for its owned components, we dodge the
server's despawn/removal logic. We may optionally combine this with spawn-time sever-side filters (Strategy F) strictly
to save bandwidth on player `Position` data, but the unyielding line of defense will be the receive markers gracefully
dumping unwanted server updates.
