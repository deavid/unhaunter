# Bevy Migration Reference for Unhaunter (0.16+)

Tbis document filters the Bevy migration guides to only include parts relevant to the Unhaunter codebase. It serves as a
quick reference for AI agents to avoid common pitfalls (e.g., using deprecated APIs or incorrect event systems).

## CRITICAL: Messages vs Events (0.17+)

Unhaunter uses a system where **buffered events** (the old `EventReader`/`EventWriter` pattern) are now called
**Messages**. The term `Event` is reserved strictly for **Observers**.

- **Buffered Communication**: Use `MessageReader<T>` and `MessageWriter<T>`.
- **Observer Communication**: Use `Trigger<T>` and `app.observe(system)`.
- **Registration**: Register messages with `app.add_message::<T>()`.
- **Derive**: Use `#[derive(bevy::prelude::Message)]` (exported via prelude in this codebase).
- **DO NOT** use `EventReader<T>` or `EventWriter<T>` for typical inter-system communication.

## ECS Changes (0.16)

### Unified Error Handling (Query::single)

`Query::single()` and `Query::single_mut()` now return a `Result` instead of panicking.

```rust
// Preferred pattern in Unhaunter
let Ok(player) = q_player.single() else { return; };
```

Similarly, `World::get_entity()` and related methods now return `Result<..., EntityDoesNotExistError>`.

### Query Ergonomics

- **Deprecated**: `Query::many()`, `Query::many_mut()`.
- **Use**: `Query::get_many()`, `Query::get_many_mut()`.
- **Renamed**: `Query::to_readonly()` is now `Query::as_readonly()`.

### Entity Manipulation

- **EntityWorldMut**: Use `EntityWorldMut` when you have `&mut World` and need to modify an entity. It is an optimized
  wrapper around the entity and world.
- **Deprecation**: `insert_or_spawn_batch` is deprecated. Use `spawn_batch` and update IDs if necessary, or use the
  `Disabled` component patterns.

### System Parameters

- **NonSendMarker**: Can be used directly as a system parameter (e.g., `fn my_system(marker: NonSendMarker)`). No need
  for `Option<NonSend<...>>`.

## Audio System (0.16)

### Volume Enum

Audio volume is now an enum. You must wrap linear values in `Volume::Linear`.

```rust
// 0.16+
audio_sink.set_volume(Volume::Linear(1.0));
// Or use decibels
audio_sink.set_volume(Volume::Decibels(0.0));
```

### AudioSink Mutation

Methods like `set_volume`, `mute`, and `unmute` now require a mutable reference to the sink.

```rust
fn adjust_volume(mut sink: Single<&mut AudioSink>) {
    sink.set_volume(Volume::Linear(0.5));
}
```

### Renames

- `AudioSinkPlayback::toggle()` is now `toggle_playback()`.

## Assets (0.16)

### Weak Handles

`Handle::weak_from_u128()` is deprecated. Use the `uuid_handle!` (0.17+) or `weak_handle!` (0.16) macro to create
handles from UUID strings.

```rust
const MY_SPRITE: Handle<Image> = uuid_handle!("b20988e9-b1b9-4176-b5f3-a6fa73aa617f");
```

## Animation (0.16)

- `EaseFunction::Steps(n)` now takes a second parameter: `EaseFunction::Steps(n, JumpAt::default())`.

## General Project Conventions

- **Edition 2024**: The codebase uses Rust Edition 2024.
- **Fallible Systems**: Systems can return `Result<(), BevyError>` (or `Result` in the project's default context).
