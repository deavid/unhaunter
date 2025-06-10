# Migration Status Report

## Current Plan Step
- Initial Build & Tackle Compiler Errors (Iterative Process)

## Work Done So Far (Summary)
- Environment setup (Rust update, Bevy system dependencies).
- `Cargo.toml` updated: Bevy to 0.16, related dependencies updated (`bevy-persistent` to 0.8.0, `bevy_framepace` to 0.19.1).
- Initial error fixing in `unsettings` (Resource trait issues) and some parts of `uncore` (some HashMap/Instant/SystemTime imports, tiled::PropertyValue fix).
- **Major Setback**: Incorrect global changes to `HashMap` and `Instant` paths (to `bevy_platform::*` then to `bevy::utils::*`) have reintroduced widespread errors. Scripting errors also corrupted some function signatures in `unstd` and `uncoremenu`.

## Next Immediate Steps
1. Correct the global paths for `HashMap`, `Instant`, `SystemTime` to their valid Bevy 0.16 locations (likely `std::collections::* ` and `std::time::*` based on compiler hints and decimation guide, pending final clarification on user feedback about Bevy-specific types).
2. Manually repair corrupted function signatures and `ChildBuilder` usage in `unstd` and `uncoremenu`.
3. Address other Bevy 0.16 API changes systematically using the new reference MD file.
