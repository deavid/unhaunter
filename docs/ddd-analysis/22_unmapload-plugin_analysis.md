# Crate Analysis: `unmapload-plugin`

## Bounded Context

**Level Initialization & Entity Orchestration**

This crate is the gateway to the gameplay world. It translates static map assets into a living ECS world by spawning
entities and populating environmental data structures.

## Responsibility

- Implement the `UnhaunterMapLoadPlugin`.
- Parse TMX map data into `BoardData` and `RoomDB`.
- Spawn the correct entities (Player, Ghost, Gear) at their designated start positions.
- Perform post-loading "Finalization" (lighting pre-bake, temperature field smoothing).
- Trigger the transition from the `Loading` state to the `InGame` state.

## Dependencies Audit

- `unassets-core` / `untiled-core`: For raw TMX and asset access.
- Almost all `un*-core` crates: To initialize their specific components.
- `unrender-std`: For character animations and sprite setups.
- `bevy`: Deeply integrated.

## Encapsulation Assessment

- **Privacy**: High. Internal systems are private.
- **Recipe Leakage**: The way specific entities (e.g., the Player) are constructed is hardcoded in this "Infrastructure"
  crate. If a ghost needs a new component, both `unghost-core` and `unmapload-plugin` must be modified.

## Semantic & Infrastructure Leakage

- **Domain Orchestration**: This crate knows "too much" about high-level game logic (e.g., how to construct recipes for
  a player vs. a ghost).
- **Map Format Coupling**: Heavily tied to Tiled (TMX) concepts. While abstracted via `untiled-core`, the loader's logic
  still reflects the layer-by-layer structure of a TMX file.

## Future Recommendations

- **Registry Pattern for Spawning**: Move toward the same pattern used in `ungear-core`. Allow plugins to register a
  "Spawner" for specific Tiled Object types. `unmapload-plugin` would then be a generic orchestrator that calls into
  these registered spawners.
- **Factory Extraction**: Move entity creation logic ("Recipes") out of the loader and into the respective domain
  plugins (e.g., an `unplayer` factory).
- **Generic Post-Processing**: Allow plugins to register "Post-Loading Tasks" so features like "Lighting Pre-bake" or
  "Ghost Influence Assignment" don't have to be hardcoded in the main loader.
