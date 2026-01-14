# DDD Analysis: `unrender-plugin`

## 1. Bounded Context

**World Rendering & Board Orchestration**. This crate implements the logic that transforms the abstract board data into
a rendered isometric scene and maintains the spatial indexing of entities.

## 2. Responsibility

- **Isometric Mapping**: Implements `apply_perspective`, which maps 3D board positions to 2D screen coordinates.
- **Board Synchronization**: Maintains the `map_entity_field` in `BoardData`, ensuring that moving entities are
  correctly indexed for spatial queries.
- **Animation Systems**: Handles sprite-sheet animations using `AnimationTimer`.
- **Resource Lifecycle**: Initializes and manages the lifecycle of core board resources like `BoardData`, `SpriteDB`,
  and `VisibilityData`.

## 3. Dependencies and Appropriateness

- **Dependencies**: `unrender-std`, `bevy`, `untypes-core`, `unspatial-core`, `unboard-core`, `unplayer-core`,
  `unmetrics-core`, `untags-core`.
- **Appropriateness**: High as a logic plugin. It depends on multiple `-core` crates because it serves as the glue
  between "what the board is" and "how the player sees it".

## 4. Encapsulation (Data/Logic Split)

- **Data**: Does not define new domain data; it mostly operates on data defined in `unboard-core` and `unrender-std`.
- **Logic**: Contains private systems for animation, spatial synchronization, and perspective application. It follows
  the "Plugin-as-Leaf-Node" pattern, exposing only `UnhaunterBoardPlugin`.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Moderate. It has code that is specifically aware of the player (to optimize spatial updates
  around them). This couples world-management logic to the player domain.
- **Infrastructure Leakage**: High (by design). It is a Bevy plugin that implements engine-specific logic like
  diagnostics and system sets.

---

## Technical Debt & Strategic Notes

- **Board vs. Render Split**: The crate is named `unrender-plugin` but its main export is `UnhaunterBoardPlugin`, which
  manages both rendering and the `BoardData` spatial index. If gameplay logic grows substantially more independent from
  rendering, the spatial indexing logic and `BoardData` ownership might need to move to a dedicated `unboard-plugin`.
- **Player Coupling**: The `sync_map_entity_field` system depends on `unplayer-core` to find the player's position. This
  could be made more generic by indexing around "Views" or "Active Regions" rather than specifically the "Player".
