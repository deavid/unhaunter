# DDD Analysis: `unsummary-core`

## 1. Bounded Context

**Mission Evaluation & Scoring Domain**. This crate owns the logic and data structures for summarizing the outcome of a
mission and calculating the player's rewards and progression.

## 2. Responsibility

- **Summary Data Model**: Defines `SummaryData`, the container for all metrics captured during a mission (time, ghosts
  found, sanity, survival).
- **Scoring Engine**: Implements the mathematical formulas for calculating base scores, applying difficulty multipliers,
  and determining mission success.
- **Financial Logic**: Calculates final money earned, insurance deposit returns, and penalty deductions.

## 3. Dependencies and Appropriateness

- **Dependencies**: `bevy`, `undifficulty-core`, `unfoundation-core`.
- **Appropriateness**: High. It depends on other domain cores (Difficulty, Foundation) to perform its evaluation.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Centralized in the `SummaryData` struct.
- **Logic**: Contains "pure" domain logic within the `SummaryData` implementation (e.g., `calculate_score`). This is a
  "Rich Domain Model" approach where the data knows the rules for its own transformation.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Low. It uses types from other domains (`GhostType`, `Grade`) but only to perform its specific
  scoring responsibility.
- **Infrastructure Leakage**: Low. It depends on Bevy for the `Resource` derive but is otherwise pure Rust logic.

---

## Technical Debt & Strategic Notes

- **Rich Model Consistency**: The scoring logic is currently well-contained. If the scoring rules become significantly
  more complex (requiring access to external registries or dynamic mod modifiers), the logic might need to be moved to
  `unsummary-plugin`.
- **Engine Readiness**: This crate is highly portable. The scoring rules are independent of rendering or input, making
  it easy to reuse in different engine implementations.
