### What Happened to `bevy::utils`?

The migration guide `bevy_utils_decimation.md` is your key resource here. `bevy_utils` used to be a grab-bag of useful types and functions. It has been "decimated" in 0.16, with its contents moved to more logical locations or spun out into new crates.

Here's a summary of the most common things you were probably using from `bevy::utils`:

*   **Collections (`HashMap`, `HashSet`)**: Moved to `bevy_platform::collections`.
*   **Hashing (`AHasher`, `FixedState`)**: Moved to `bevy_platform::hash`.
*   **Time (`Instant`, `Duration`)**: Moved to `bevy_platform::time` or `core::time`.
*   **Logging Macros (`info!`, `warn!`, `error!`)**: These are now re-exported from `bevy_log`.
*   **`BoxedFuture`**: Moved to `bevy_tasks`.
*   `bevy::utils::SystemTime` is gone. You should now import and use `std::time::SystemTime` directly.


This is due the no_std support that was added, the original functions can't be in bevy::utils anymore.

### ChildSpawnerCommands is now ChildSpawnerCommands

When you use the with_children method on EntityCommands, the closure now receives a &mut ChildSpawnerCommands instead of a &mut ChildSpawnerCommands.

### Despawn changes

.despawn is gone,.despawn is obsolete.

The old.despawn() is the new despawn()
