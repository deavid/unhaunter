# Migration Changes Explained

This file will document the rationale behind significant or potentially confusing migration changes.

## `bevy::utils` Decimation and Type Paths (Ongoing Discussion)
- Initial changes to use `std::collections::HashMap` and `std::time::Instant` were made based on compiler suggestions and some migration notes.
- User feedback indicates a preference for Bevy-specific versions (`bevy::utils::HashMap`, `bevy::utils::Instant`) for performance and WASM compatibility.
- The `bevy_utils_decimation.md` guide in `bevy_migration.txt` states these types moved to `bevy_platform` or `std`. This caused confusion.
- Current Attempt: Reverted to `bevy::utils::*` paths as per strong user feedback, but this caused unresolved/private errors, indicating these paths might not be valid in Bevy 0.16 as direct replacements. The correct Bevy 0.16 paths for these need to be definitively established. The last attempt to use `bevy_platform::*` was also incorrect.
