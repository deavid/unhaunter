# DDD Analysis: `unplayer-core`

## 1. Bounded Context

The **Core Player Data** context. This crate defines the fundamental state of a player entity without implementing the
complex gameplay rules that govern that state. It serves as the data contract for any system interacting with the
player.

## 2. Responsibility

- **Data Definition**: Defines the components (`GhostSprite`, `StaminaStat`, `Inventory`, `HeldObject`, `ActivePlayer`)
  and resources (`PlayerConfig`, `PlayerAnimationData`) that constitute a player.
- **Cross-Crate Sharing**: Provides a common type-safe interface for input systems, the game engine, and the UI to read
  and write player state.

## 3. Dependencies and Appropriateness

- **Dependencies**: `bevy` (ECS fundamentals), `unfoundation-core` (gear types), `unsettings-core` (controls),
  `unspatial-core` (positioning).
- **Appropriateness**: High. It avoids gameplay-heavy dependencies like `unboard-core` or `unrender-std`. However, it
  has become a bucket for diverse concepts (Inventory, Stats, Input Buffer, and Global Game Config).

## 4. Encapsulation (Data/Logic Split)

- **Logic Leakage (High)**: Contrary to the "Data-Only" goal for `-core` crates, there is significant gameplay logic
  embedded here:
  - **`StaminaStat`**: Contains a full state machine including frame-rate-dependent math, depletion/recovery rates, and
    state transitions (`exhausted`).
  - **`Sanity/Health` Method Logic**: Includes mathematical formulas for damage and recovery. These are gameplay rules
    (rules of the "Unhaunter" universe) and do not belong in a data-only crate.
  - **`Inventory` Helper Methods**: While `add_item` is a simple factory, the `InventoryNext` logic starts to cross the
    line into behavior management.

## 5. Semantic & Infrastructure Leakage

- **Semantic Confusion**: `GhostSprite` is misnamed. Semantically, it represents the "Player Character" or "Player
  Stats". The word "Sprite" implies a visual/infrastructure dependency (which it technically doesn't have, but the name
  is misleading).
- **Infrastructure Coupling**: `HeldObject` couples to Bevy's `Entity` ID for `hiding_spot`, which is standard for Bevy
  but technically binds data lifecycle to the ECS engine.
- **Board Isolation**: Successfully isolated from `unboard-core`.

---

## Technical Debt & Strategic Notes

- **Critical Violation**: `StaminaStat` logic must be moved to `unplayer-plugin`.
- **Naming Debt**: `GhostSprite` should be renamed to something like `PlayerCharacter` or `PlayerStats` to avoid
  confusion with actual "Ghost" entities or "Sprite" visuals.
- **Externalization**: Player stats (speed, drain rates) should eventually be moved to a configuration file rather than
  being hardcoded in methods.
