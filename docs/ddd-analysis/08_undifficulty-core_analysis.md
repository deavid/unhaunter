# Crate Analysis: `undifficulty-core`

## Bounded Context

**Gameplay Difficulty & Configuration**

This crate translates the user's high-level difficulty selection (e.g., "Professional") into concrete parameters that
influence gameplay (ghost speed, player health, temperature drop rates).

## Responsibility

- Define the `DifficultySettings` trait and the `DifficultySettingsData` composite struct.
- Map `Difficulty` enum variants (from `untypes-core`) to specific gameplay values.
- Manage `CurrentDifficulty` as a global resource.
- Provide data for the "Manual" (tutorial pages) based on the selected difficulty.

## Dependencies Audit

- `untypes-core`: For the `Difficulty` enum.
- `unfoundation-core`: For basic domain types.
- `bevy`: Standard resource management.

## Encapsulation Assessment

- **Privacy**: The heavy mapping logic is hidden in private modules, exposing a clean trait-based interface.
- **Refinement**: Successfully decouples the _act_ of choosing a difficulty from the _meaning_ of that difficulty.

## Semantic & Infrastructure Leakage

- **Minimal Leakage**: It is a core domain logic crate.
- **Coupling**: Tied to the "Manual" system, which is a specific gameplay feature, but appropriate here as difficulty
  and tutorials are linked.

## Future Recommendations

- **Dynamic Difficulty**: If the game ever needs moddable or dynamically adjusted difficulty, the currently hardcoded
  mappings in `impl_difficulty.rs` should be moved to an external config file (e.g., RON/JSON) loaded via
  `unassets-core`.
