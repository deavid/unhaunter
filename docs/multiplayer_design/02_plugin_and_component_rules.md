# The Domain Triad: Plugin & Component Rules

- **Status:** Core Architectural Directive
- **Date:** April 2026

## 1. The Core Problem: The Single-Player Shortcut

Historically, multiplayer bugs in Unhaunter (like the "Ghost scales when hit by repellent in single-player, but not in
multiplayer" bug) stem from **Topology Bleed**. Developers and AI agents write systems that conflate _Authorship_ (who
decides what happens) with _Representation_ (who draws what happened).

In a single-player mindset, `GhostHitByRepellent` directly mutates `Transform.scale`. In a networked game, this is a
fatal error. The Server computes the hit, mutates its local `Transform.scale`, and the Client (who actually draws the
ghost) never sees it because `Transform` isn't replicated over the network.

**The Fix is Structural:** We do not rely on developers remembering to add `With<Remote>`, `run_if(Authority)`, or
manually syncing transforms. We make invalid data flows **syntactically illegal** to write.

We enforce a strict Acyclic Dependency Graph (DAG) for data flow: **Skeleton/Gameplay → Decoration/Presentation**

---

## 2. The Domain Triad Architecture

To enforce the DAG without creating a 100+ crate explosion, every feature domain (e.g., `ghost`, `player`, `vitals`)
must be grouped in a directory (e.g., `crates/ghost/`) and strictly divided into three crates.

These crates represent the physical boundaries of the ECS.

### 1. `unX-core` (The Schema / Vocabulary)

The absolute truth of what the domain _is_. It contains pure data.

- **Contents:** Components (`GhostSprite`, `RepellentSwelling`), Events (`GhostSpawnRequest`), Enums.
- **Allowed:** `bevy_ecs`, `serde`, primitive math crates, other `-core` dependencies.
- **BANNED:** `bevy_app`, `bevy_replicon`, `bevy_transform`, `bevy_sprite`, `bevy_render`, `bevy_audio`.
- **Rule:** This crate does not know the network exists. It does not know the screen exists. It defines the nouns of the
  game.

### 2. `unX-logic` (The Arbiter / Server Authority)

The brain of the domain. It advances the simulation, moves entities, and decides outcomes.

- **Contents:** Systems, Plugin definitions, `bevy_replicon` registration (`app.replicate::<T>()`).
- **Deployed On:** Dedicated Server, Host, Join Client. (All peers load this, but execution is fenced by
  `run_if(Authority)`).
- **Network:** COMPLETELY NETWORK AWARE. It registers the types for encoding/decoding.
- **Allowed:** `unX-core`, `bevy_app`, `bevy_ecs`, `bevy_replicon`.
- **BANNED:** `bevy_transform`, `bevy_sprite`, `bevy_render`, `bevy_audio`, `unrender-std`, `unrender-plugin`.
- **Rule:** Because `Transform` and `Sprite` are explicitly banned from `Cargo.toml`, you _cannot_ write a system that
  scales a ghost visually. You are forced to write the state to a `-core` component (e.g., `RepellentSwelling`).

### 3. `unX-presentation` (The Lens / Client Illusion)

The puppet. It senses the state of the world and makes it visible/audible.

- **Contents:** Visuals, Audio, `Transform` syncing, UI draw calls, Particle Effects.
- **Deployed On:** Host, Join Client, Offline. (**NEVER** loaded on Dedicated Server).
- **Network:** COMPLETELY NETWORK UNAWARE. It does not know what `Remote` or `LocallyOwned` means. It blindly draws
  whatever the `-core` components dictate.
- **Allowed:** `unX-core`, `bevy_transform`, `bevy_sprite`, `bevy_render`, `bevy_audio`, `unrender-plugin`.
- **BANNED:** Querying `&mut` on any gameplay state component.
- **Rule:** Presentation is strictly **Read-Only** for gameplay state. It reads `RepellentSwelling.amount` and applies
  it to `&mut Transform.scale`.

---

## 3. The Rules of Application

### Rule A: The Hydration Pattern (Spawning Entities)

Because Logic cannot access visual components, spawning is permanently split into two phases.

**Phase 1: Naked Spawning (Server / Logic)** The Arbiter `unghost-logic` listens to an intent (like `GhostSpawnRequest`)
and spawns a "naked" entity containing only pure data and network markers:

```rust
commands.spawn((
    GhostSprite { aggression: 0.5 },
    Position::new(10, 10),
    Replicated, // Handled by replicon
));
```

**Phase 2: Hydration (Client / Presentation)** The Lens `unghost-presentation` listens for the arrival of new logical
entities (either spawned locally or received mid-game over the network) and "clothes" them:

```rust
fn hydrate_ghost_visuals(
    mut commands: Commands,
    q_new_ghosts: Query<(Entity, &Position), Added<GhostSprite>>
) {
    for (entity, pos) in q_new_ghosts.iter() {
        commands.entity(entity).insert((
            SpriteBundle { ... },
            Transform::from_translation(pos.to_world_coords()),
            // Add AudioSinks, LerpPosition, etc.
        ));
    }
}
```

_Why this is mandatory:_ This inherently solves late-join multiplayer. A player joining 40 minutes late receives the
`GhostSprite` component via Replicon. Local `Added<GhostSprite>` triggers, and the ghost immediately materializes on
their screen. No catch-up code required.

### Rule B: State over Ephemeral Events

If an action needs to cause a visual change across the network (e.g., a ghost blowing out a lightbulb), you **must not**
rely on local Bevy `Event`s crossing the logic-presentation boundary on the server. Instead, Logic mutates a continuous
`-core` state component (e.g., `LightFixture { blown_out: true }`). Presentation detects the `Changed<LightFixture>`
component and plays the glass-shatter sound locally.

### Rule C: The Raycast / Input Exception (Backwards Flow)

Because Presentation owns `Transform`, Camera, and coordinates, **Input Evaluation** belongs in the Presentation layer.
When a user clicks the screen:

1. `unX-presentation` casts a ray against `Transform`s to find what entity/tile was clicked.
2. It translates this into an **Intent Event** (e.g., `InteractAt(Entity)`).
3. The Intent Event is routed over the network (Client → Server via Replicon's ClientEvent).
4. `unX-logic` receives the `InteractAt` event, evaluates if it's legal, and updates the canonical simulation data.

---

## 4. Enforcement (How we stop bugs)

This architecture drastically reduces context windows. To implement a visual feature, you only need to look at
`unX-core` and `unX-presentation`.

To physically prevent regressions:

- `-logic` crates MUST NOT have `bevy_transform`, `bevy_sprite`, or `bevy_render` in their `Cargo.toml`.
- If a developer or AI Agent complains they "cannot find Transform in unghost-logic," **do not import it.** They are
  solving the problem in the wrong layer. They must push data to a component inside `unX-core`.
