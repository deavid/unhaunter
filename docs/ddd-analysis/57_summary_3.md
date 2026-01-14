# DDD Analysis Summary #3: Player, Rendering, and Persistent Infrastructure

## Overview

This third and final batch of analysis covered the player domain, the rendering pipeline, persistent user data
(profiles/settings), and the specialized infrastructure for maps (Tiled) and communication (Walkie-Talkie).

## Key Findings

### 1. Architectural Strengths

- **Marker-Based Decoupling**: `untags-core` and `unui-core` provide a centralized vocabulary for entity roles and UI
  anchors. This allows systems (like Ghost AI or the Walkie-Talkie) to find targets (like "the player" or "the evidence
  UI") without depending on the heavy internal structures of those modules.
- **Rich Domain Logic (Walkie)**: `unwalkie-core` successfully encapsulates complex prioritization and pacing logic for
  narration in a way that is mostly independent of the Bevy engine.
- **Persistence Consistency**: The `unsettings` and `unprofile` crate pairs follow a consistent and clean pattern for
  separating domain data from I/O infrastructure, making them highly stable and easy to maintain.
- **Spatial Math Isolation**: `unspatial-core` provides a robust mathematical foundation for the isometric world,
  keeping the coordinate mapping logic centralized.

### 2. Identified Semantic Leakage & Violations

- **Logic Leakage in Core**: Several `-core` crates have drifted from the "Data-Only" goal:
  - `unplayer-core`: Contains embedded stamina tick logic and health/sanity methods.
  - `unprofile-core`: Contains the mathematical formula for leveling up (`xp_to_level`).
  - `unnoise-core`: Houses the heavy precomputation and coordinate mapping math for Perlin noise.
- **Visual/Infrastructure Leakage**:
  - `unspatial-core`: The `to_screen_coord` method and `global_z` field leak rendering/perspective details into the
    "pure" spatial domain.
  - `unpicking-core`: Exposes rendering-specific concepts like `alpha_threshold`.
  - `unroot-plugin`: Acts as a "God Plugin" for assets, maintaining a centralized list of file paths for every domain in
    the game.
- **Semantic Confusion**: `unplayer-core` uses the name `GhostSprite` to represent the player character's data, which is
  misleading in a game where "ghosts" are the primary antagonist.

### 3. "Ghost Engine" Portability Score

- **Player & Progression**: 7/10 (Data structures are solid, but mechanics are currently hard-coded in core).
- **Spatial & Math**: 8/10 (Highly reusable math, but needs to shed rendering hacks).
- **Persistent Systems**: 9/10 (Excellent separation of concerns).
- **World Rendering**: 6/10 (Deeply tied to 2D isometric sprite-layering techniques).

## Strategic Recommendations

- **Formalize "Plugin-Only" Logic**: Conduct a dedicated refactor to move stamina ticks, XP calculations, and noise
  generation into the `-plugin` layer. Core crates should return to being pure data containers.
- **Decentralize Asset Management**: Transition from the monolithic `GameAssets` in `unroot-plugin` to a registry where
  individual plugins declare and manage their own assets.
- **Refine Spatial Boundaries**: Move screen-coordinate projection and visual layering (`global_z`) from
  `unspatial-core` to `unrender-std` to ensure the spatial domain remains engine-agnostic.
- **Clarify Player Domain**: Rename `GhostSprite` and similar types to reflect their actual role (e.g.,
  `PlayerCharacter`) to reduce cognitive load for new developers.
