# DDD Analysis: `unsettings-plugin`

## 1. Bounded Context

**Settings Persistence Infrastructure**. This crate bridges the domain data in `unsettings-core` with the physical
storage/filesystem.

## 2. Responsibility

- **Persistence Orchestration**: Uses `bevy_persistent` to ensure that settings are automatically loaded from and saved
  to disk.
- **Platform Abstraction**: Resolves the correct system-specific path for configuration files (e.g., using `dirs` to
  find the user's config folder).
- **Serialization Handling**: Configures the serialization format (RON) and error recovery (reverting to defaults).

## 3. Dependencies and Appropriateness

- **Dependencies**: `unsettings-core`, `bevy`, `bevy_persistent`, `dirs`.
- **Appropriateness**: High. It acts as a clean infrastructure layer that isolates the rest of the game from file I/O
  and path resolution details.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Does not define new domain data; it wraps core settings in `Persistent<T>` containers.
- **Logic**: Contains the imperative logic for file path resolution and `bevy_persistent` builder configuration.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Low. It only knows the names of the settings files (e.g., `video_settings.ron`).
- **Infrastructure Leakage**: High (by design). It is deeply coupled with the filesystem and the specific persistence
  library choice.

---

## Technical Debt & Strategic Notes

- **Centralized Config Path**: It hardcodes the application name `unhaunter-game` for path resolution. In a generic
  "Ghost Engine," this would be a parameter.
- **Consistency with Profile**: This crate follows a pattern very similar to `unprofile-plugin`, maintaining
  architectural consistency across different persistence needs.
