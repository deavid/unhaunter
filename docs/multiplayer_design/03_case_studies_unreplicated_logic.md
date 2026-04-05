# Case Studies in Topology Bleed: Analyzing Tween, Locked, and GhostInfluence

- **Status:** Architectural Case Studies
- **Date:** April 2026

When auditing a Bevy multiplayer codebase against the **Domain Triad Architecture** (Logic vs. Presentation separation),
existing components often reveal deep-seated design flaws.

The question isn't just "Should this be replicated?" The real questions are: _What is this component? Who owns it? And
is it lying about its identity?_

Here is an analysis of three problematic components in the Unhaunter codebase: `Tween`, `Locked`, and `GhostInfluence`,
evaluating them strictly against the Triad Rules.

---

## Case 1: `Tween` — The Event-Substitute Anti-Pattern

### The Current Design

`Tween` is currently treated as a logic component but is intentionally _not_ replicated. Instead, the server logic emits
a custom network event (`HostMovableMotionEvent`/`MovableMotionBroadcast`), and the client reconstructs a local `Tween`
from that event to animate things like opening doors.

### The Verdict: OUTRIGHT WRONG (Topology Bleed)

This design violates the core DAG constraint: **Skeleton/Gameplay → Decoration/Presentation**. `Tween` is a
frame-by-frame float interpolator. It is the definition of visual ephemera.

**Why it fails:** The server logic is dictating _how_ a door animates to the client via an event, rather than dictating
_what the state of the door is_. This is a single-player shortcut masked as a network event.

**How it should be designed:**

1. **Logic Component (`unX-core/src/components/logic/`):** Create a replicated `DoorState { is_open: bool }`. The Server
   mutates this. It does not know what a "Tween" is.
2. **Presentation Component (`unX-core/src/components/presentation/`):** `Tween` belongs strictly in the Presentation
   folder. It is never replicated, and logic never imports it.
3. **The Bridge:** The Client's Presentation plugin detects `Changed<DoorState>`. When a door flips from closed to open,
   Presentation structurally _inserts_ a local `Tween` onto the entity to animate the `Transform`. No custom network
   events required; state drives presentation.

---

## Case 2: `Locked` — The Transient State Trap

### The Current Design

`Locked` is currently treated as non-replicated, transient interaction state. Its lifecycle seems tied to local elapsed
time (animations), meaning the network tolerates the client not knowing about it immediately.

### The Verdict: FLAWED DESIGN (Conflated Authority)

This is the classic "Transient State" trap. If a component is acting as both a gameplay blocker and a visual animation
trigger, it will cause late-join bugs.

**Why it fails:** If `Locked` dictates that a player _cannot physically open a door_ (Gameplay Authority), it absolutely
must be replicated. If a player joins the server 5 seconds after a ghost locks a door, the client must receive that
`Locked` state immediately. If it's skipped because it's "just a short interaction state," the client's local simulation
will incorrectly allow them to try and interact with the door.

**How it should be designed:**

1. **Logic Component:** `Locked { expires_at_tick: u64 }` (or similar). This is authoritative gameplay state. It **must
   be replicated**. (Note: We use absolute timestamps/ticks, not `Timer`s, for replicated logic).
2. **Presentation Component:** If the door "jiggles" while locked, we create a `JiggleAnimation` component in the
   Presentation folder.
3. **The Bridge:** When the client user tries to interact with the door, the Client Presentation reads the replicated
   `Locked` component. Seeing it is locked, Presentation spawns a local `JiggleAnimation` to show the user it failed.

---

## Case 3: `GhostInfluence` / `SpectralInfluence` — The Invisible Glow Bug

### The Current Design

`GhostInfluence` tracks the "charge" a ghost leaves on an object (e.g., to make it glow under UV light). However, it is
not replicated. `SpectralInfluence` is the downstream presentation component that actually drives the shaders. It is
also not replicated.

### The Verdict: LATENT MULTIPLAYER DESYNC

This is a fatal break in the data pipeline. The Server computes the charge, but the Client draws the glow. If the charge
never crosses the network, the UV lights break for anyone who isn't the host.

**Why it fails:** It relies on the assumption that "only the server needs the influence math," completely forgetting
that the client's renderer uses that math as an external input to draw UV signatures. Because neither component
traverses the network, the visual state is orphaned on the server.

**How it should be designed:**

1. **Logic Component (`GhostInfluence`):** This is authoritative, long-lived gameplay state. The Ghost creates it. It
   **MUST BE REPLICATED**. This ensures late-joiners instantly see the room's UV heatmaps correctly.
2. **Presentation Component (`SpectralInfluence`):** This dictates exactly how the object interacts with the renderer
   (UV capacity, color shifting, alpha). It **MUST NOT BE REPLICATED**.
3. **The Bridge:** Client-side Presentation observes `Changed<GhostInfluence>` (which arrived via Replicon) and runs a
   system mapping the logic data (`charge_value`) onto the local presentation data (`SpectralInfluence.uv_charge`). The
   shader then reads `SpectralInfluence` to render the glow.

---

## Conclusion

If a component in the `logic/` module is not replicated, ask yourself: _Does the client ever need to see the result of
this?_

- If **Yes** (e.g., `GhostInfluence` -> UV Glow), **it must be replicated**.
- If **No** (e.g., a hidden AI pathfinding heatmap), **it is a valid Server-Only exception.**
- If **It's just an animation** (e.g., `Tween`), **it doesn't belong in the logic folder in the first place.**
