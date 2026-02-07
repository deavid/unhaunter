# Deep Dive: Ghost Hunt Visuals and Player Damage in Multiplayer

## 1. Research Overview

Investigation into why the ghost doesn't turn red during hunts on the client, why clients don't perceive damage, and why
the host seems to take damage instead (including when in the truck).

## 2. Findings (Facts)

### Ghost Visuals (Red Tint)

- **The Component Sync Chain:**
  - The `GhostSprite` component's `hunt_warning_active` and `hunt_warning_intensity` fields are synced via `GhostState`
    network messages in [crates/unnet-core/src/messages.rs](crates/unnet-core/src/messages.rs#L56).

> **RESEARCH FINDING:** In [crates/unnet-core/src/messages.rs](crates/unnet-core/src/messages.rs#L62-L68), the
> `GhostState` struct is missing `pub hunt_target: bool`. Consequently,
> [crates/unnet-plugin/src/systems.rs](crates/unnet-plugin/src/systems.rs#L125) never applies the `hunt_target` status
> to client-proxies.

- **CRITICAL MISSING FIELD:** The `hunt_target` (which indicates an active hunt) is **NOT** included in the `GhostState`
  struct.
- On the Client, `unnet-plugin` updates the `GhostSprite` but `hunt_target` remains default/false.
- `unghost-plugin` runs `ghost_visual_sync` on both Host and Client, mapping `GhostSprite.hunt_target` to
  `Ethereal.hunt_target` in
  [crates/unghost-plugin/src/systems/visual_sync.rs](crates/unghost-plugin/src/systems/visual_sync.rs#L13).
- The rendering system in `unlight-plugin`
  ([crates/unlight-plugin/src/maplight/systems/sprites.rs](crates/unlight-plugin/src/maplight/systems/sprites.rs#L233))
  uses `ethereal.hunt_target` to decide whether to apply the red tint color. Since this is never true on the Client, the
  ghost stays in "spectral white" mode during hunts.

- **Inverted Warning Lerp:**
  - In
    [crates/unlight-plugin/src/maplight/systems/sprites.rs](crates/unlight-plugin/src/maplight/systems/sprites.rs#L246),
    the warning color lerps to `RED` based on `warning_intensity`.

> **RESEARCH FINDING:** The lerp logic in
> [crates/unlight-plugin/src/maplight/systems/sprites.rs](crates/unlight-plugin/src/maplight/systems/sprites.rs#L246) is
> `color.lerp_to(RED, (1.5 - ethereal.warning_intensity))`. In
> [crates/unghost-plugin/src/systems/ghost_ai/enrage.rs](crates/unghost-plugin/src/systems/ghost_ai/enrage.rs#L174), the
> intensity **increases** from 0.5 to 1.0. This math causes the ghost to start more red and become cleaner/white as the
> hunt approaches, which is the exact inverse of the intended visual progression.

### Player Damage & Health

- **No Health Sync:** `PlayerSprite.health` and `PlayerSprite.sanity` are **NOT** included in the `PlayerState` network
  message in [crates/unnet-core/src/messages.rs](crates/unnet-core/src/messages.rs#L35).
- **Authoritative Damage (Host-Only):** Ghost damage is applied in `handle_hunting_phase`
  ([crates/unghost-plugin/src/systems/ghost_ai/enrage.rs](crates/unghost-plugin/src/systems/ghost_ai/enrage.rs#L257)),
  which is called by `ghost_enrage`. This system runs **only on the Host**.

> **RESEARCH FINDING:** The `handle_hunting_phase` logic in
> [crates/unghost-plugin/src/systems/ghost_ai/enrage.rs](crates/unghost-plugin/src/systems/ghost_ai/enrage.rs#L260)
> queries `Query<(&mut PlayerSprite, &Position), Without<GhostSprite>>`. It lacks a `Without<InTruck>` filter, allowing
> the ghost to damage players who have retreated to the truck if the ghost is close enough to the truck's boundary (or
> if the ghost wanders near).

- **Client Perception:**
  - The `visual_health` system runs on both Host and Client, updating the UI based on the local `PlayerSprite`
    component.
  - Since the Host does not sync health decreases to the Client, the Client's local `MainPlayer` entity never sees its
    health drop in its own UI.
  - The Host **does** see the Host player's health drop because it happens locally.

### "Attacking the Ghost" & Host Damage

- **Repellent Logic:**
  - When a Client uses a repellent, it triggers locally and sends an interaction/input to the Host.
  - The Host processes this and spawns `RepellentParticle` entities on the Host's side at the Client player's position.
  - These particles hit the Host's ghost, increasing `ghost.rage`.
  - Increased rage triggers hunts.
- **Why the host is hurt:**
  - During a hunt, the ghost targets a player randomly (or based on distance).
  - If the Host is targeted, they lose health.
  - Because the Host's health bar is the only one "moving" (from the perspective of being synced to the local player),
    it feels like the Host is being punished for the Client's actions.
- **Truck Damage:**
  - The hunt target selection in
    [crates/unghost-plugin/src/systems/ghost_ai/movement.rs](crates/unghost-plugin/src/systems/ghost_ai/movement.rs#L143)
    and damage application in `enrage.rs` **do not check if the player is in the truck** (`InTruck` component).

> **RESEARCH FINDING:** In
> [crates/unghost-plugin/src/systems/ghost_ai/movement.rs](crates/unghost-plugin/src/systems/ghost_ai/movement.rs#L143),
> the `evaluate_weighted_distance` function iterates over all players to find a target. It does not filter out players
> with the `InTruck` component, meaning the ghost can start "hunting" a player who is technically safe, leading to the
> ghost hovering near the truck or dealing damage through the door.

## 3. Theory & Guesses

- The "attacking the ghost" report might be a combination of the Client player successfully hitting the ghost (which
  makes it angry) and the resulting hunt damaging the Host player because they are the only ones observing their health
  bar drop.
- The Ghost might be "clumping" damage or choosing the Host player more frequently due to how `q_player.iter()` is
  ordered or how distances are handled when multiple players exist.
- The red tint failure is likely a direct result of the missing `hunt_target` bool in the networking struct.

## 4. Proposed Investigation Next Steps

1. Verify the `GhostState` and `PlayerState` message definitions to ensure they align with the intended multiplayer sync
   goals (should health/sanity be synced?).
2. Check if `ghost_visual_sync` should be running on the host and syncing its results, or if the and missing network
   field is enough.
3. Investigate adding `Without<InTruck>` filters to ghost targeting and damage systems.
4. Confirm if `warning_intensity` lerp is indeed inverted in `unlight-plugin`.
