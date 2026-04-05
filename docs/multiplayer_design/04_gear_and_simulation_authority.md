# The Driver Architecture: Multiplayer Gear Rules

- **Status:** Core Architectural Directive
- **Date:** April 2026

## The Core Axiom: The Server is a Router, Clients are the Engine

In Unhaunter, the Server is intentionally kept CPU-light. It **does not** simulate heavy environmental grids (Miasma,
Thermal, Sound) and it **does not** simulate gear logic. The Server is simply a fast, dumb blackboard that holds
canonical state, validates authority, and redistributes data to solve NAT and sync issues.

Because the Server is blind to the environment, **Clients must simulate everything.** Every client evaluates the grids
to make the world _appear_ continuous. However, to prevent 4 clients from applying the same damage to the Ghost
simultaneously, we must strictly separate **Physical Possession** from **Simulation Authority**.

---

## Rule 1: The Driver Mandate

Every piece of Active Gear in the game **must have a Designated Driver at all times**, regardless of whether it is in a
pocket or on the floor.

- **The Driver:** The single client authorized to run the mathematical simulation for an item and push state mutations
  to the Server. Identified by the `Owner(ClientId)` component on the Server, and the `LocallyOwned` marker on the
  Client.
- **Physical Possession:** Where the item is. Identified by `EquipmentPosition`, `PlayerGear` inventory slots, or
  `FloorItemCollidable`.

**Do not conflate these two concepts.** An item on the floor is _physically_ unpossessed, but it is still mathematically
Driven by a client so its battery can drain and it can interact with the Ghost.

---

## Rule 2: The Drop and Grab Dance

Because Possession and Drivership are decoupled, the lifecycle of picking up and dropping items follows strict rules:

- **Dropping Gear:** When a player drops an item, the physical state changes (add `FloorItemCollidable`, change to
  `EquipmentPosition::Deployed`). **DO NOT remove `Owner` or `LocallyOwned`.** The player who dropped it remains the
  Designated Driver. They continue to calculate its battery drain and grid collisions while it sits on the floor.
- **Grabbing Gear:** Grab-ability is defined _purely_ by physical state (`With<FloorItemCollidable>`). It has nothing to
  do with `Owner`.
  - When Player B grabs an item on the floor (currently Driven by Player A), Player B sends a `RequestGrab` to the
    Server.
  - The Server validates the physical grab, forcefully strips `Owner(Player A)`, and assigns `Owner(Player B)`.
  - Player A naturally loses `LocallyOwned` and stops driving. Player B naturally gains `LocallyOwned` and starts
    driving. There is never a race condition.

---

## Rule 3: The Orphan Catcher (Fault Tolerance)

Gear must never be Owned/Driven by a client who is disconnected or sitting in the unloaded Lobby state. A Driver must be
in `SimulationState::Ready`. If a Driver disconnects or leaves the mission, the Server must immediately execute the
**Orphan Catcher**:

1. Find all gear entities where `Owner == Exiting_Client`.
2. Reassign the `Owner` token of all those entities to a surviving, `Ready` client.
3. The surviving client seamlessly takes over the mathematical simulation for those items. _(Note: This transfers
   network authority, not physical inventory. The items physically remain where they were)._

---

## Rule 4: The 3-Tier Component Stratification

To prevent ad-hoc network spaghetti, every single piece of gear must be strictly divided into three conceptual tiers.
There are no exceptions for specific items.

### Tier A: The Skeleton (External / Canonical State)

- **What it is:** Discrete, binary, or stepped states (e.g., `FlashlightStatus`, `QuartzStoneData.cracks`,
  `RepellentFlask.qty`).
- **Who owns it:** The Server holds the canonical truth and replicates it reliably to all clients.
- **Who mutates it:** ONLY the Designated Driver (`With<LocallyOwned>`). The Driver sends state-change requests to the
  Server via `ExportClientComponent<T>`.

### Tier B: The Internal Sim (Analog / Fuel State)

- **What it is:** Continuous, analog values that power the item (e.g., `Battery.level`,
  `QuartzStoneSkin.energy_absorbed`).
- **Who owns it:** Mutated ONLY by the Designated Driver running local grid math.
- **How it syncs:** Pushed to the Server continuously (eventually consistently) via Unreliable channels. _(Note: Must be
  flagged for future optimization/throttling)._

### Tier C: The Presentation (The Illusion)

- **What it is:** UI text, Sprites, Audio, Particles (e.g., `StatusText`, `GearSprite`).
- **Who owns it:** **EVERY CLIENT, ALWAYS.**
- **How it works:** Client A, B, and C all read the replicated Skeleton data and the Internal Sim data, and
  independently calculate how to draw the item on their own screens every frame. Presentation components are **never**
  sent over the network. They are deterministic illusions based on Tier A and Tier B data.
