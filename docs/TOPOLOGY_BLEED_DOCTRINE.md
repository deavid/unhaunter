# TOPOLOGY BLEED DOCTRINE

- Status: Core Architectural Directive
- Scope: Design Philosophy + ECS Domain Manifesto + Multiplayer Execution Rules
- Date: April 2026

This document is the hard boundary policy for network-safe ECS development in Unhaunter.

If a design allows gameplay authority and visual representation to leak into each other, the design is wrong. If a crate
can mutate what it should only observe, the crate boundary is wrong. If correctness depends on developers remembering a
filter, a run condition, or a replication footnote, the system is fragile by construction.

The fix is not discipline. The fix is architecture that makes bad flows difficult or impossible to write.

---

## 1) The Failure Pattern We Are Banning

The classic bug:

- Logic computes a valid gameplay outcome on the server.
- A visual component (for example `Transform.scale`) is mutated directly in that logic path.
- Clients never receive the visual mutation because the visual component is not replicated.
- Single-player appears fine. Multiplayer diverges.

This is Topology Bleed: authorship and representation mixed in one system.

---

## 2) The Non-Negotiable Data Flow

Authoritative state flows one way:

Skeleton/Gameplay -> Decoration/Presentation

- Logic decides and writes canonical state.
- Presentation reads canonical state and renders local illusion.
- Presentation may emit player intent upstream, never authoritative outcomes.

---

## 3) The Domain Triad (Per Feature)

Every feature domain (ghost, player, vitals, gear, etc.) is split into three crates.

### `unX-core` (Schema)

- Owns pure data vocabulary: components, events, enums.
- Knows nothing about rendering, transforms, audio, or replication wiring details.
- Defines what the domain is, not how it is simulated or displayed.

### `unX-logic` (Authority)

- Owns simulation rules and outcomes.
- Registers replicated gameplay state (`app.replicate::<T>()`).
- Deployed On: Dedicated Server, Host, Join Client (Execution is fenced by `run_if(Authority)`).
- Network-aware, presentation-blind.
- Must not depend on visual crates/types (`Transform`, sprite/render/audio stacks).

### `unX-presentation` (Lens)

- Owns visuals/audio/local polish.
- Consumes gameplay state and materializes it to local experience.
- Deployed On: Host, Join Client, Offline (NEVER loaded on Dedicated Server).
- Network-unaware and gameplay-write-forbidden.
- Must not mutate authoritative gameplay components.

---

## 4) Hard Rules

### Rule A: Two-Phase Spawning (Hydration)

- Logic spawns naked authoritative entities (gameplay components + replication markers).
- Presentation hydrates those entities when gameplay components appear (`Added<T>` pattern).

#### Phase 1: Naked Spawning (Server / Logic)

```rust
commands.spawn((
    GhostSprite { aggression: 0.5 },
    Position::new(10, 10),
    Replicated, // Handled by replicon
));
```

#### Phase 2: Hydration (Client / Presentation)

```rust
fn hydrate_ghost_visuals(
    mut commands: Commands,
    q_new_ghosts: Query<(Entity, &Position), Added<GhostSprite>>
) {
    for (entity, pos) in q_new_ghosts.iter() {
        commands.entity(entity).insert((
            SpriteBundle { ... },
            Transform::from_translation(pos.to_world_coords()),
        ));
    }
}
```

Why: late join works by default. Replicated gameplay state appears, hydration fires locally, entity materializes with no
special catch-up path.

### Rule B: State Over Ephemeral Cross-Layer Events

- If a visual consequence must be consistent across peers, logic writes durable gameplay state.
- Presentation reacts to `Changed<T>` on that state.
- Do not depend on local server-side transient events to create remote visual effects.

### Rule C: Input/Raycast Exception (Allowed Backflow)

- Presentation performs coordinate/raycast evaluation (it owns camera/transform context).
- Presentation emits intent events.
- Logic validates intent and writes canonical outcomes.

### Rule D: Component Intent Must Be Physical

Inside `unX-core/src/components/`, components must be physically segregated by intent:

- `logic/*`: authoritative domain state. **MUST BE REPLICATED** (`Position`, `GhostSprite`) unless it is explicitly
  pure server-side internal calculations (like AI brain state pathfinding).
- `presentation/*` or `local/*`: local ephemera and visual process state. **MUST NOT BE REPLICATED**
  (`InteractionParticle`, `MotionBlur`).

No mixed dumping in one generic component file for convenience.

Timer classification rule:

- Gameplay cooldown/authority timing -> logic.
- Visual fade/animation lifetime -> presentation/local.

### Rule E: Access Matrix (Enforced Ownership)

- Logic components:
  - `unX-logic`: read + write.
  - `unX-presentation`: read-only.
- Presentation/local components:
  - `unX-logic`: no access.
  - `unX-presentation`: read + write.

---

## 5) Cargo-Level Enforcement

Architecture is enforced in dependencies, not PR comments.

- `-logic` crates must not include transform/sprite/render/audio dependencies.
- If a logic implementation needs `Transform`, the implementation is in the wrong crate.
- Move intent into a gameplay component in `unX-core`; let `unX-presentation` project it visually.

**For AI Agents and Developers:**
If you complain that you "cannot find `Transform` in `unghost-logic`," DO NOT import it. You are solving the problem in
the wrong layer. Push data to a State component inside `unX-core`.

---

## 6) Review Gate (Reject Conditions)

Reject changes when any of the following is true:

- Logic directly mutates visual components.
- Presentation mutates authoritative gameplay state.
- A feature needs manual multiplayer exceptions to remain correct.
- Components with mixed intent are placed in a single unsegregated module.
- A crate boundary is bypassed because "it was faster".

---

## 7) Practical Litmus Test

Before merging any feature, answer all three:

1. Can late-join reconstruct visuals from replicated gameplay state alone?
2. Can dedicated server run logic without any presentation dependency?
3. Can presentation be deleted and reimplemented without changing gameplay rules?

If any answer is no, Topology Bleed still exists.

---

## 8) Relationship To Existing Philosophy Docs

This doctrine is an executable extension of:

- design philosophy (high-level product and architecture intent)
- ECS domain manifesto (ownership, boundaries, signal direction)

Use this file as the boundary contract during implementation and code review when multiplayer correctness is at stake.
