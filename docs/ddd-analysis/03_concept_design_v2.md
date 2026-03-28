# Concept Design v2

This document is a refined version of the Unhaunter Architectural Manifesto. It removes completed milestones and sharpens the focus on the remaining architectural debt required to achieve **Context Isolation, Emergent Simulation, and Low Cognitive Load.**

---

## Phase 1: The Bedrock (Physics & Fields)

The "Stage" is a set of interacting 3D matrices. While the crates exist, they are not yet fully "blind."

- **`unworld-physics-grids` (T1):** Thermal, Sound, EMF, Visibility, Miasma.
  - **The Goal:** They must be blind. They compute diffusion/propagation math exclusively.
  - **The Debt:** Currently, some fields (like `unthermal`) still import high-level logic like `undifficulty-core` or `unbehavior-core`.
  - **The Fix:** Invert the relationship. Fields should not know about "Difficulty." Instead, the Mission/Difficulty domain should push "Ambient Temperature" or "Diffusion Rate" signals into the fields.

## Phase 2: The Agent Slices (Breaking the Nouns)

We continue to drain `unplayer` and `unghost` into modular vertical feature-backpacks.

- **`unvitals-plugin` (T1):** The sole arbiter of biological math (Health, Sanity, Stamina).
  - **The Debt:** It currently "pulls" data by querying `unthermal`, `unsoundfield`, and `unghost` to compute drain. This is a "Tell, Don't Ask" (TDA) violation.
  - **The Fix:** Vitals should listen for `Stimulus` signals (e.g., `ColdStimulus`, `FearStimulus`) emitted by the fields and the ghost. Vitals computes the result; it doesn't hunt for the cause.
- **Logistics vs. Sensing (The Gear Split):**
  - **The Debt:** We still have an implementation split (`ungear` vs `ungearitems`).
  - **The Goal:** Split by concern instead:
    - **Logistics:** The handoff protocol (Hand -> Backpack -> World). Moves Entity IDs. Doesn't know what an EMF reader is.
    - **Sensing:** Entities that sample a field and produce a reading. This is the "business logic" of the equipment.

## Phase 3: The Metagame (Mission vs. Career)

We have successfully separated `unmission` and `uncareer`, and moved the "Ledger" (`unsummary-core`) to Tier 2. The remaining work is decoupling the economy.

- **`uncareer-plugin` (T2):** The Metagame Engine.
  - **The Goal:** It should be the only place that knows about "Bank Accounts," "XP Curves," and "Map Unlocking."
  - **The Handoff:** It waits for the `MissionSummary` event from the Mission domain. It reads the report, calculates the payout, and updates the save-file. The Mission domain should never touch a dollar sign.

---

## Appendix: The Horizontal-Cut Problem (Rendering)

While the UI horizontal-cut (`unui-core`) has been resolved, `unrender-std` remains a "shared bag" that contains simulation state mis-filed as presentation.

### `unrender-std`: Simulation State Mis-filed as Presentation

`unrender-std` lives at Tier 4, yet it contains components that describe the simulation, not the rendering pipeline. This forces Tier 1 domain crates to import Tier 4.

| Component                                                 | Target Domain                      | Why it belongs there                                                                                                                                                                |
| --------------------------------------------------------- | ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `CharacterAnimationState` / `CharacterAnimationDirection` | `unlocomotion-core`                | Locomotion computes the state; rendering merely observes it to pick a frame. |
| `SpriteLayer`                                             | `unboard-core` or `unspatial-core` | Draw-ordering is a function of spatial depth in the simulation. |
| `LightSensitive`                                          | `unlight-core`                     | This is an ontology input for the light field, similar to `ThermalEmitter`. |
| `GameSprite` / `MapTileSprite`                            | `unboard-core` / Identity          | These are identity markers for the simulation. |

**The Fix:** Move these components to their respective `-core` crates. `unrender-std` should be drained until it only contains GPU-facing types, shaders, and asset handle collections.
