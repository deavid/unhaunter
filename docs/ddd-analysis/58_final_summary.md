# Master Architectural Debt Report

## 1. Critical Priority: Domain Integrity Violations

_Issues where crates violate the fundamental "Core = Data / Plugin = Logic" architecture rule._

| Crate                    | Issue Description                                                                                                                                                         | Violation Type    |
| :----------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | :---------------- |
| **`unplayer-core`**      | **Logic in Core:** Contains active state-machine logic for `StaminaStat` (ticking, exhaustion state transitions) and mathematical formulas for Health/Sanity calculation. | **Logic in Core** |
| **`unplayer-core`**      | **Naming Conflict:** The struct `GhostSprite` is defined here but semantically represents the **Player Character**, creating a misleading domain model.                   | **Semantic**      |
| **`unnoise-core`**       | **Logic in Core:** Contains heavy mathematical initialization logic for noise tables and coordinate mapping algorithms.                                                   | **Logic in Core** |
| **`unprofile-core`**     | **Logic in Core:** Contains the mathematical formula for player leveling (`xp_to_level`) within the data definition crate.                                                | **Logic in Core** |
| **`uninteraction-core`** | **Logic in Core:** The `InteractiveStuff` struct contains imperative logic for sound emission and material manipulation.                                                  | **Logic in Core** |
| **`unnavigation-core`**  | **Logic in Core:** The `CollisionHandler` contains complex collision math and waypoint queue advancement logic.                                                           | **Logic in Core** |
| **`unboard-core`**       | **Logic in Core:** The `NPCStranger` component contains mutable gameplay state (`seen` flag) and logic for parsing Tiled properties.                                      | **Logic in Core** |

---

## 2. High Priority: Engine Agnosticism Blockers

_Issues that couple the core architecture to specific rendering perspectives (2D) or impede the extraction of a generic
engine._

| Crate                | Issue Description                                                                                                                                                                          | Impact                 |
| :------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :--------------------- |
| **`unspatial-core`** | **Visual Coupling:** The fundamental `Position` struct contains a `to_screen_coord` method, coupling the math layer to 2D isometric projection.                                            | **3D Pivot Blocker**   |
| **`unspatial-core`** | **Visual Coupling:** The `Position` struct contains a `global_z` field, which is a rendering-specific hack for 2D sprite layering.                                                         | **3D Pivot Blocker**   |
| **`unboard-core`**   | **Hardcoded Features:** The board data structure contains explicit fields for specific gameplay features like "Miasma" indices, preventing generic usage.                                  | **Engine Scalability** |
| **`unassets-core`**  | **Monolithic Design:** The crate contains explicit references to every specific game asset (icons, manual pages), functioning as a static content list rather than an infrastructure tool. | **Modding Blocker**    |
| **`unroot-plugin`**  | **Hardcoded Loader:** The asset loading process is defined as a hardcoded imperative list of files, requiring recompilation to add or remove assets.                                       | **Modding Blocker**    |

---

## 3. Medium Priority: Cross-Domain Coupling

_Issues where distinct domains have direct dependencies on each other's internal logic._

| Crate                | Issue Description                                                                                                                                                      | Leakage Type              |
| :------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :------------------------ |
| **`untruck-plugin`** | **High Coupling:** Depends on nearly every other core crate in the workspace (`unghost`, `unplayer`, `ungear`, `unboard`, etc.), creating a "God Plugin."              | **Architectural**         |
| **`unlight-plugin`** | **Domain Leakage:** directly modifies ambient audio volume based on visibility calculations.                                                                           | **Light $\to$ Audio**     |
| **`ungear-core`**    | **Context Coupling:** The `GearStuff` system parameter bundles dependencies for Board, Ghosts, and Difficulty, coupling the Gear domain to all of them simultaneously. | **Context Overload**      |
| **`unmenu-core`**    | **Domain Leakage:** Generic UI templates accept specific `PlayerProfile` data structures as input arguments.                                                           | **UI $\to$ Data**         |
| **`ungame-plugin`**  | **Tech Coupling:** Gameplay systems explicitly trigger low-level technical functions like `rebuild_lighting_field`.                                                    | **Gameplay $\to$ Render** |
| **`unrender-std`**   | **Domain Leakage:** The `SpriteType` enum contains variants for specific gameplay entities (`Ghost`, `Player`, `Miasma`).                                              | **Render $\to$ Gameplay** |

---

## 4. Low Priority: Implementation & Infrastructure

_Hardcoded values and specific implementation details embedded in the wrong layer._

| Crate                   | Issue Description                                                                                                                   | Type              |
| :---------------------- | :---------------------------------------------------------------------------------------------------------------------------------- | :---------------- |
| **`unmanual-plugin`**   | **Content Hardcoding:** Manual pages, text, and layout are defined directly within Rust closures.                                   | **Content**       |
| **`unpicking-core`**    | **Magic Numbers:** Contains hardcoded constants for rendering-specific values like `alpha_threshold` (0.8) and `tile_size` (16x16). | **Configuration** |
| **`unnpc-plugin`**      | **Magic Numbers:** Contains hardcoded heuristic constants for interaction timers and proximity distances.                           | **Configuration** |
| **`unsettings-plugin`** | **Hardcoded Identity:** The application name `unhaunter-game` is hardcoded for file path resolution.                                | **Configuration** |
| **`unmetrics-core`**    | **Global State:** Relies on a `static` global transport mechanism rather than ECS resources.                                        | **Pattern**       |
| **`ungear-plugin`**     | **Stale Code:** Contains unused files (`evidence_systems.rs`) and potentially duplicate logic (`gear_stuff.rs`).                    | **Hygiene**       |
