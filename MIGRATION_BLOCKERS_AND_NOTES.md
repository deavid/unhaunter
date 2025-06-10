# Migration Blockers and Notes

## Current Blockers/Points of Confusion
1.  **Definitive Type Paths for Bevy 0.16**: Significant confusion remains regarding the correct, idiomatic paths for types previously in `bevy::utils` like `HashMap` and `Instant`. User feedback prioritizes Bevy-specific versions for performance/WASM, but migration notes and compiler errors often point to `std::*` or the (apparently incorrect for direct use) `bevy_platform::*` paths. This needs to be the first thing clarified and fixed globally.
2.  ** Usage in `fn` Signatures**: The pattern `fn foo(parent: &mut ChildBuilder, ...)` used in `unstd` and `uncoremenu` is problematic as `ChildBuilder` (now `bevy::hierarchy::ChildBuilder`) is a `SystemParam`. This likely requires a refactor of how these UI/manual drawing functions are structured or called. Direct use in `fn` pointers might not be viable.
3.  **Code Corruption by Scripts**: Previous automated attempts with `perl`/`sed` have malformed some source files (e.g., function signatures, incorrect `use` statements). These need careful manual (simulated) correction.

## Assumptions Made
- `bevy-persistent 0.8.0` and `bevy_framepace 0.19.1` are the correct Bevy 0.16-compatible versions. This seems to hold so far.

## Notes
- The migration is proving slower than anticipated due to the cascading effects of initial incorrect changes and the complexity of fixing them across a large workspace.
- A more cautious, crate-by-crate or even file-by-file approach for syntax and type resolution is needed after global reversions are done.
