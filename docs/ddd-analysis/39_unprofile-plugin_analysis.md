# DDD Analysis: `unprofile-plugin`

## 1. Bounded Context

**Persistence Infrastructure**. This crate bridges the domain data in `unprofile-core` with the physical
storage/filesystem.

## 2. Responsibility

- **I/O Management**: Initializes and manages the `UserProfile` resource, saving and loading from disk.
- **Platform Abstraction**: Determines platform-specific storage paths (e.g., using `directories` for native config
  folders).
- **Session Recovery**: Implements "recovery" logic, such as `apply_insurance_deposit`, which ensures game state
  consistency across sessions.

## 3. Dependencies and Appropriateness

- **Dependencies**: `unprofile-core`, `bevy_common_assets` (for simplifying save/load), `directories` (path resolution),
  `serde_ron`.
- **Appropriateness**: High. It acts as a clean infrastructure layer.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Does not define new domain data; it wraps core data in persistence-aware containers.
- **Logic**: Contains all the "impure" logic: file path resolution, error handling for I/O, and Bevy startup systems for
  loading profiles.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Moderate. It has some knowledge of game-specific concepts like "insurance deposits" to perform
  recovery logic. In a stricter architecture, this "recovery" logic might be triggered by a gameplay plugin that uses
  the profile plugin as a tool.
- **Infrastructure Leakage**: High (by design). It is the infrastructure layer, focusing on paths and serialization
  formats.

---

## Technical Debt & Strategic Notes

- **Persistence Decoupling**: Currently tied to RON. If the game moves to a binary format or cloud saves, this is the
  crate that would change.
- **Logic Migration Target**: This crate is the appropriate home for the leveling logic currently residing in
  `unprofile-core`.
