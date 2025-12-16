# Component: A Rigorous Abstract Definition

## The Problem

Without a clear definition, "component" becomes meaningless. People will suggest:

- "Love is a component"
- "Strawberry is a component"
- "Motor skills is a component"

These are all wrong, but without rigorous criteria, you can't explain _why_.

---

## The Definition

> **A Component is a discrete, measurable property that can be attached to an entity, has a representable state, and can
> be queried by systems to determine behavior.**

---

## The Five Tests

For something to qualify as a Component, it must pass **ALL** of these:

| Test                 | Question                                                                  | Failure Example                                                                        |
| -------------------- | ------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| **1. Attachable**    | Can it belong to a specific entity/object in the simulation?              | "Love" fails — love is a _relationship between_ entities, not a property _of_ one.     |
| **2. Representable** | Can its state be encoded as data (numbers, enums, flags, structs)?        | "Motor skills" fails — it's an emergent capability from multiple systems, not a value. |
| **3. Queryable**     | Can a system ask "which entities have this?" and get a meaningful answer? | "Strawberry" fails — that's an entity (a thing), not a property of a thing.            |
| **4. Separable**     | Can it be added, removed, or changed independently of other properties?   | "Existence" fails — it's not separable; removing it means removing the entity.         |
| **5. Observable**    | Does it have a current value that can be read at any moment?              | "Potential" fails — it describes what _could_ happen, not what _is_.                   |

---

## Applying the Tests

| Candidate        | Attachable?        | Representable?       | Queryable?                    | Separable?    | Observable?       | Verdict                        |
| ---------------- | ------------------ | -------------------- | ----------------------------- | ------------- | ----------------- | ------------------------------ |
| **Position**     | ✓ Entity has one   | ✓ Vec3               | ✓ "All entities at X"         | ✓ Can change  | ✓ Current coords  | ✅ Component                   |
| **Health**       | ✓ Entity has HP    | ✓ Number             | ✓ "All entities with HP < 50" | ✓ Can lose it | ✓ Current value   | ✅ Component                   |
| **Temperature**  | ✓ Tile has it      | ✓ Float              | ✓ "All tiles above 30°"       | ✓ Changes     | ✓ Current reading | ✅ Component                   |
| **Love**         | ✗ Between entities | —                    | —                             | —             | —                 | ❌ Relationship, not component |
| **Strawberry**   | ✗ IS an entity     | —                    | —                             | —             | —                 | ❌ Entity, not component       |
| **Motor skills** | ✗ Emergent         | ✗ No single value    | —                             | —             | —                 | ❌ Capability, not component   |
| **Scary**        | ✗ Subjective       | ✗ To whom?           | —                             | —             | —                 | ❌ Perception, not property    |
| **Walking**      | ✗ It's an action   | ✗ Process, not state | —                             | —             | —                 | ❌ Behavior, not component     |

---

## What Components Are NOT

| Category               | Why it's not a component    | What it actually is                   |
| ---------------------- | --------------------------- | ------------------------------------- |
| **Entities**           | A thing that HAS components | "Ghost", "Player", "Door"             |
| **Relationships**      | Connects two entities       | "Haunting", "Friendship", "Parent-of" |
| **Behaviors**          | Actions/processes           | "Walking", "Hunting", "Flickering"    |
| **Emergent qualities** | Result of system logic      | "Scary", "Dangerous", "Winning"       |
| **Capabilities**       | What systems enable         | "Can move", "Can attack"              |
| **Events**             | Instantaneous happenings    | "Door opened", "Ghost spawned"        |
| **Intentions**         | What an agent wants         | "Wants to kill player"                |

---

## The Litmus Test (Quick Version)

When someone proposes a component, ask:

> **"Show me the struct. What fields does it have? What values can those fields hold right now?"**

If they can't answer that, it's not a component.

### Examples

- **Position**: `{ x: f32, y: f32, z: f32 }` ✅
- **Love**: "Uh... it's a feeling?" ❌
- **Strawberry**: "It's... a strawberry?" ❌ (That's an entity with components like `Edible`, `Position`, `Sprite`)

---

## Component vs. Other ECS Concepts

```
┌─────────────────────────────────────────────────────────────────┐
│                         WORLD                                   │
│                                                                 │
│  ┌─────────────────┐                                            │
│  │     ENTITY      │  ← A unique ID, nothing more               │
│  │   (Ghost #42)   │                                            │
│  │                 │                                            │
│  │  ┌───────────┐  │                                            │
│  │  │ Component │  │  ← Position { x: 5.0, y: 3.0 }             │
│  │  └───────────┘  │                                            │
│  │  ┌───────────┐  │                                            │
│  │  │ Component │  │  ← GhostType::Banshee                      │
│  │  └───────────┘  │                                            │
│  │  ┌───────────┐  │                                            │
│  │  │ Component │  │  ← Aggression { level: 0.7 }               │
│  │  └───────────┘  │                                            │
│  └─────────────────┘                                            │
│                                                                 │
│  ┌─────────────────┐                                            │
│  │     SYSTEM      │  ← Reads components, produces behavior     │
│  │  (HuntingAI)    │     "If Aggression > 0.5, chase player"    │
│  └─────────────────┘                                            │
│                                                                 │
│  ┌─────────────────┐                                            │
│  │     EVENT       │  ← Instantaneous signal                    │
│  │ (GhostSpawned)  │     "Ghost #42 appeared at (5,3)"          │
│  └─────────────────┘                                            │
│                                                                 │
│  ┌─────────────────┐                                            │
│  │    RESOURCE     │  ← Global state, not per-entity            │
│  │  (GameTimer)    │     "Current mission time: 4:32"           │
│  └─────────────────┘                                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## For Unhaunter Specifically

### Valid Components (pass all five tests)

| Component   | Representation              |
| ----------- | --------------------------- |
| Position    | Board coordinates (x, y, z) |
| LightLevel  | Float 0.0–1.0               |
| Temperature | Float in Celsius            |
| SoundLevel  | Float (dB or normalized)    |
| EMFReading  | Float                       |
| Sanity      | Float 0–100                 |
| GhostType   | Enum of all ghost types     |
| RoomID      | Integer/Handle              |
| Visibility  | Bool or float               |
| Battery     | Float 0.0–1.0               |
| Equipped    | Bool                        |

### NOT Components (but exist in the game)

| Not-a-component  | What it actually is                                                  |
| ---------------- | -------------------------------------------------------------------- |
| Ghost            | **Entity** (has Position, GhostType, etc.)                           |
| Player           | **Entity** (has Position, Sanity, Inventory, etc.)                   |
| Hunting          | **Behavior state** (part of AI system logic)                         |
| Evidence         | **Event/discovery** (something happened, maybe stored in a resource) |
| Scary atmosphere | **Emergent** (result of light + sound + presence combined)           |
| "Is dangerous"   | **Query result** (ask system: "is this entity dangerous right now?") |
| Walking          | **Action** (system is executing movement over time)                  |

---

## Edge Cases and Clarifications

### "But love COULD be a float 0.0–1.0!"

True, but then you're not modeling "love" — you're modeling **affinity** or **relationship strength**. And that's a
component on a _relationship entity_, not on a person.

```
Entity: Relationship #99
  - ParticipantA: Player #1
  - ParticipantB: NPC #7
  - Affinity: 0.8        ← THIS is the component
```

### "Isn't 'IsHunting' a component?"

It could be, as a **marker component** (a component with no data, just presence/absence):

```rust
struct IsHunting;  // Marker component
```

This passes the tests:

- Attachable: Yes, to a ghost entity
- Representable: Yes, presence = true, absence = false
- Queryable: Yes, "all entities currently hunting"
- Separable: Yes, can add/remove it
- Observable: Yes, either it's there or not

### "What about 'Velocity'? It describes motion."

Velocity is a **component** because it's observable state right now:

```rust
struct Velocity { x: f32, y: f32, z: f32 }
```

"Walking" is NOT a component because it's a process/behavior. But `Velocity` (the current speed vector) IS a component.

---

## Summary

| If someone proposes... | Ask them...                                                   |
| ---------------------- | ------------------------------------------------------------- |
| A noun                 | "Is that an entity or a property of an entity?"               |
| An adjective           | "Can you give it a numeric or enum value? Observable when?"   |
| A verb                 | "That's a behavior, not state. What's the STATE it produces?" |
| A feeling              | "Between whom? Measured how? Stored where?"                   |
| Something vague        | "Show me the struct definition."                              |

The struct test is the ultimate filter. If it can't be a struct with fields that have values, it's not a component.
