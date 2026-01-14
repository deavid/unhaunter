# DDD Analysis: `unprofile-core`

## 1. Bounded Context

**Player Progression & Identity**. This crate serves as the definition for a player's persistent state (money, XP,
statistics).

## 2. Responsibility

- **Data Definition**: Defines the `UserProfile` schema and its sub-components (`ProgressionData`).
- **Initialization**: Provides default state initializers for new profiles.

## 3. Dependencies and Appropriateness

- **Dependencies**: `bevy` (ECS), `serde` (serialization), `unfoundation-core` / `untypes-core`.
- **Appropriateness**: High. It follows the standard pattern for a data-carrying core crate.

## 4. Encapsulation (Data/Logic Split)

- **Logic Leakage (Moderate)**:
  - **`xp_to_level`**: This crate contains the mathematical formula for player leveling based on experience points.
  - **Violation**: According to the "Data-Only" core principle, this calculation belongs in the plugin or a separate
    gameplay-math utility, as it defines a game mechanic rather than just a data structure.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Low. It uses standard domain types.
- **Infrastructure Leakage**: Low. While it depends on `serde`, it does not implement any I/O or storage logic.

---

## Technical Debt & Strategic Notes

- **Logic-in-Core**: Move `xp_to_level` and similar progression formulas to `unprofile-plugin` or a shared math utility.
  The core should only hold the `xp` and `level` values.
- **Schema Evolution**: As a core crate, changes here reflect in the saved data format. It should remain stable to avoid
  breaking player saves.
