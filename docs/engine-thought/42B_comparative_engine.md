# Unhaunter: The Comparative Engine Analysis (Subsystem Edition)

## 1. Executive Summary

**Unhaunter** is a **Nightmare Simulator**. To build it, we need to identify the architectural **Subsystems** required
to support its gameplay loop.

This report analyzes 25 reference games to isolate the "Big Chunks" of functionality—the Subsystems—that the engine must
provide.

---

## 2. The Subsystem Standard

To prevent "Feature Soup," all proposed ideas are subjected to the **Crate Test**:

> **"Could this be a standalone crate?"**

If yes, it is a Subsystem. If no, it is likely a component or system belonging to a larger Subsystem.

---

## 3. Layer Assignment Rules

| Layer          | Rule                                                                                    | Scope                                  |
| :------------- | :-------------------------------------------------------------------------------------- | :------------------------------------- |
| **ENGINE**     | Must be **genre-agnostic**. Could make a Sci-Fi Stealth game without changing the code. | Spatial, Physics, I/O, Networking.     |
| **SHARED MOD** | Essential to **"Paranormal Investigation"** but agnostic to specific mission type.      | Evidence, Sanity, Ghost Concepts.      |
| **GAME MODE**  | Specific to the **win/loss loop** of a single game type.                                | Classic Mode Rules, Escape Mode Rules. |

---

## 4. Thematic Cluster Analysis (Subsystem Lens)

We analyzed reference games to find the underlying machinery.

### Cluster A: The Physical World (Space & Physics)

| Game                       | Key Insight                                          | Derived Subsystem                 | Layer  |
| :------------------------- | :--------------------------------------------------- | :-------------------------------- | :----- |
| **Phasmophobia**           | Light and sound are the primary feedback mechanisms. | **Sensory Physics** (Light/Sound) | Engine |
| **Pools**                  | The space itself is the antagonist.                  | **Spatial** (Grid/Map)            | Engine |
| **Liminal Shroud**         | Atmosphere (fog, grain) defines the mood.            | **Atmosphere**                    | Engine |
| **The Mortuary Assistant** | Temperature changes indicate presence.               | **Thermodynamics**                | Engine |

### Cluster B: The Actor & Interaction

| Game                 | Key Insight                               | Derived Subsystem | Layer  |
| :------------------- | :---------------------------------------- | :---------------- | :----- |
| **Devour**           | Carrying items is a burden.               | **Inventory**     | Engine |
| **Dead by Daylight** | Interaction requires time and commitment. | **Interaction**   | Engine |
| **Lethal Company**   | Movement is physical (stamina, weight).   | **Locomotion**    | Engine |

### Cluster C: The Paranormal Layer

| Game               | Key Insight                                 | Derived Subsystem        | Layer      |
| :----------------- | :------------------------------------------ | :----------------------- | :--------- |
| **Phasmophobia**   | Sanity determines vulnerability.            | **Sanity**               | Shared Mod |
| **Demonologist**   | Evidence gathering leads to identification. | **Forensics** (Evidence) | Shared Mod |
| **Ghost Watchers** | The ghost has a hidden identity/type.       | **Entity Identity**      | Shared Mod |

---

## 5. The Unhaunter Subsystem Inventory

### 5.1 Engine Layer (Genre-Agnostic)

These are the foundational crates.

| Subsystem           | Responsibilities                                                | Crate Name (Hypothetical) |
| :------------------ | :-------------------------------------------------------------- | :------------------------ |
| **Spatial**         | Grid coordinates, transforms, directions, map loading.          | `unspatial`               |
| **Sensory Physics** | Light propagation, sound propagation, visibility calculation.   | `unsensory` / `unlight`   |
| **Thermodynamics**  | Heat propagation, thermal mass, cooling/heating.                | `unthermal`               |
| **Locomotion**      | Actor movement, collision, pathfinding basics.                  | `unactor`                 |
| **Inventory**       | Slots, item containers, pickup/drop logic, encumbrance.         | `uninventory`             |
| **Interaction**     | Raycasting, "Use" verbs, hold-to-interact, toggleables (doors). | `uninteraction`           |
| **Atmosphere**      | Fog, post-processing, volumetric effects per zone.              | `unatmosphere`            |
| **Fluid Dynamics**  | (Optional) Miasma flow, gas spread.                             | `unfluid`                 |

### 5.2 Shared Mod Layer (Paranormal Standard)

These define the "Ghost Hunting" genre.

| Subsystem           | Responsibilities                                                                 | Crate Name (Hypothetical) |
| :------------------ | :------------------------------------------------------------------------------- | :------------------------ |
| **Sanity**          | Mental health tracking, drain modifiers, hallucination triggers.                 | `unsanity`                |
| **Forensics**       | Evidence types (EMF, Freezing), detection logic, journal tracking.               | `unforensics`             |
| **Entity Identity** | Ghost types, names, ages, death causes, hidden traits.                           | `unidentity`              |
| **Gear**            | Specific tool implementations (EMF Reader, Thermometer) using Engine subsystems. | `ungear`                  |

### 5.3 Game Mode Layer (Specific Loops)

These define the specific game being played.

| Subsystem           | Responsibilities                                                     | Crate Name (Hypothetical) |
| :------------------ | :------------------------------------------------------------------- | :------------------------ |
| **Ghost AI**        | State machines, hunting logic, roaming behavior (Classic vs Escape). | `unghost`                 |
| **Mission Control** | Win/loss conditions, objectives, progression.                        | `unmission`               |
| **Ritual**          | (Escape Mode) Steps to banish/bind the entity.                       | `unritual`                |

---

## 6. Conclusion

The architecture is no longer a soup of components. It is a set of **Subsystems**.

**The Engine** is a simulation of:

- Space (`unspatial`)
- Physics (`unsensory`, `unthermal`)
- Stuff (`uninventory`, `uninteraction`)

**The Game** is a layer of meaning applied to that simulation:

- "High `unthermal` reading" = "Freezing Temperatures Evidence" (`unforensics`)
- "Low `unsanity`" = "Ghost initiates Hunt" (`unghost`)
