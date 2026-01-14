# DDD Analysis: `unsettings-core`

## 1. Bounded Context

**User Preference Domain**. This crate owns the definition of all user-configurable parameters, ranging from technical
hardware settings to gameplay preferences.

## 2. Responsibility

- **Schema Definition**: Defines the data structures for `VideoSettings`, `AudioSettings`, `GameplaySettings`, and
  `ControlSettings`.
- **Persistence Support**: Provides `serde` implementations for all settings to allow saving/loading from disk.
- **Reflection Support**: Uses `strum` and `enum-iterator` to allow UI systems to dynamically list and modify settings.

## 3. Dependencies and Appropriateness

- **Dependencies**: `bevy`, `serde`, `enum-iterator`, `strum`.
- **Appropriateness**: High. It is a pure-data core crate that provides the foundational types for the settings
  subsystem.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Contains only structs and enums with basic derive macros.
- **Logic**: Zero logic. No Bevy systems or complex methods.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Low. It defines gameplay concepts like `MovementStyle` (Isometric vs. Orthogonal), which are
  inherent to the game's configuration.
- **Infrastructure Leakage**: Low. While it depends on Bevy for `Resource` and `Component` derives, it remains largely
  engine-agnostic in its data representation.

---

## Technical Debt & Strategic Notes

- **Moddability**: This crate is highly stable. Adding new settings involves adding fields here, which is then picked up
  by the UI and persistence logic.
- **Clean Architecture**: This is a model example of a clean core crate.
