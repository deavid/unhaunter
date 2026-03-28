# Unhaunter ECS Domain Manifesto

This document defines the architectural philosophy of Unhaunter. It is written to guide new feature development,
structure bug fixes, and serve as the ultimate rubric for code reviews.

We are building a highly decoupled, Entity Component System (ECS) game in Bevy. Our primary enemies are **spaghetti
code**, **unnecessary rebuilds (Death by a Thousand Crates)**, and **massive context windows** (for both humans and AI).

To defeat them, we enforce strict Domain Isolation.

---

## ⚖️ The "More Of / Less Of" Matrix

| 🟢 We Want MORE Of...                                                                                         | 🔴 We Want LESS Of...                                                                                            |
| :------------------------------------------------------------------------------------------------------------ | :--------------------------------------------------------------------------------------------------------------- |
| **Vertical Slices:** A domain owning everything from its internal logic to how it renders and plays audio.    | **Horizontal Trashcans:** Dumping types into `shared_types` or `common` just to bypass circular dependencies.    |
| **Capabilities (Verbs/Adjectives):** Generic tags like `EmitsEmf`, `RepelsSpirits`, `PsychologicalPressure`.  | **Identities (Nouns):** Checking if an entity `With<GhostMarker>` or `if item == Gear::Crucifix`.                |
| **The Stage as a Blackboard:** Domains leaving "Facts" on the physical Board (T1a) for other domains to find. | **Direct System Queries:** Domain A querying Domain B's internal component to figure out what Domain B is doing. |
| **Domain-owned Spawning:** The Ghost plugin explicitly building the Ghost entity based on blind requests.     | **God Systems:** A central `setup_level` or `replicon` system that knows the exact components of everything.     |
| **State as Distinct Components:** `IsHunting`, `IsStunned`, `IsFleeing` added/removed dynamically.            | **God Enums for State:** `enum GameState { Normal, Hunting, Fleeing }` matched across the whole codebase.        |
| **Pushing Signals:** Emit an event or drop a component when something happens.                                | **Telepathy (Pulling):** Reading another system's internal state to guess if you should play a sound.            |

---

## 🏛️ The 4 Pillars of Domain Isolation

### 1. The Trashcan Fallacy & Vertical Slice Supremacy

**The Problem:** When two domains need the same piece of data, the temptation is to extract that data into a "common"
T0/T1 crate. This creates "Trashcan Crates" that own no logic, bloat dependencies, and act as a magnet for junior devs
to dump unrelated code. **The Rule:** A piece of domain knowledge belongs strictly inside its domain. Do not cut
horizontally. If Domain A needs to influence Domain B, they must communicate via **Events** (for discrete actions) or
**Stage Primitives** (for continuous state), not by sharing private structs.

### 2. Absolute Data Ownership (The Spawning God)

**The Problem:** Infrastructure crates (like `unreplicon-core` for network) or orchestrators (like `unmission`) end up
importing every `-core` crate in the workspace because they need to spawn entities over the network or at map load. They
become God Crates. If a component changes, the network breaks. **The Rule:** Only the domain knows how to build its own
entities.

- The Network should just emit generic `NetworkHydrateRequest` events.
- The Map Loader should just leave `SpawnRequest("Type")` markers.
- The domain (e.g., `unghost-plugin`) listens to these blind requests, owns all the "Know-How" about what components are
  needed, and acts on them.

### 3. The Blackboard & Primitive Speech (Actors Don't Touch Actors)

**The Problem:** The Ghost needs to break the Thermometer and lower the Player's Sanity. If the Ghost queries the
Thermometer and Player directly, the domains become permanently welded together. **The Rule:** Actors touch the World;
the World touches Actors. Use the physical stage (T1a: `unboard`, `unthermal`, `unfog`) as a Blackboard.

- The Ghost domain writes a generic `TemperatureDelta` and `Miasma` at its location.
- The Thermometer blindly reads `TemperatureDelta` from the board.
- The Vitals system blindly reads `Miasma` from the board. Neither the Ghost, Vitals, nor Thermometer need to know the
  others exist.

### 4. The Telepathy Test

**The Problem:** The UI or Audio layer wants to look cool, so it queries `GhostHuntState` to play a spooky heartbeat.
You have now leaked gameplay logic into the presentation layer. If ghost mechanics change, audio breaks. **The Rule:**
Do not read another actor's mind. Use **Tell, Don't Ask**. If the ghost is angry, the ghost must push a
`PlayAudioRequest(SpookyRoar)` onto itself or the world. The Audio system blindly fulfills requests without knowing
_why_ the request was made.

---

## 🛠️ Practical Guide for the Team

### For Juniors Writing New Code

1. **Adding a Feature:** Limit your work to ONE bounded context `-core` and `-plugin` pair. If your AI or your brain
   tells you to import 5 other crates to get the job done, **stop**. You are crossing boundaries.
2. **Context Window:** You should only need to send the AI your specific domain files, plus maybe the shared primitives
   from T1a (if you are writing to the world).
3. **Adding Items/Gear:** Never write `if target.has::<Type>()` inside your gear logic. Define what your gear _does_ via
   a primitive capability component (e.g. `LightOverride`) and attach that component to the entity.

### For Juniors Fixing Bugs

1. **Finding the Bug:** Because of data ownership, bugs belong to the domain that owns the data. If sanity is draining
   too fast, the bug is _only_ in `unvitals-plugin` (misreading the board) OR in the domain dropping the Miasma fact
   (e.g., `unghost-plugin`). You never have to search the networking or orchestrator crates.
2. **Don't Add Spillage:** To fix a bug, do not add a new edge to the graph (e.g. do not add a
   `Query<&OtherDomainMarker>` just to patch an edge case). Fix the signal being emitted.

### For Code Reviews and Audits

Code reviewers must enforce isolation strictly. Look for these exact "smells":

- **The Upward/Lateral Import:** A `-plugin` or `-core` importing another `-core` from the same or higher tier (except
  for generic T1a stage primitives). _Reject._
- **The Vocabulary Violation:** Seeing `With<GhostMarker>` inside anything other than `unghost` crates. _Reject._ (Tell
  them to use a generic Capability component).
- **The Orchestrator Build:** Seeing `.insert(GhostBundle { ... })` inside a network, map, or mission orchestrator
  crate. _Reject._ (Tell them to emit a spawn request and let the Ghost plugin handle it).
- **The Trashcan PR:** A PR moving 5 components into `uncommon` or `unshared` so they can be accessed globally.
  _Reject._ (Tell them to use the Blackboard or Events).
