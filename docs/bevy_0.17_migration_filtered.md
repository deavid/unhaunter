# Bevy Migration Reference for Unhaunter (0.17+)

This document filters the Bevy 0.17 migration guide to only include parts relevant to the Unhaunter codebase.

## Messages vs Events (0.17+)

This split is now fully enforced.

- **Messages**: Buffered communication using `MessageReader<M>` and `MessageWriter<M>`. These are used for most
  inter-system communication in Unhaunter.
- **Events**: Specifically for **Observers** (Observable Events). Use `Trigger<E>` in systems that observe them.
- **Renames**:
  - `EventReader` -> `MessageReader`
  - `EventWriter` -> `MessageWriter`
  - `World::send_event` -> `World::write_message`
  - `Commands::send_event` -> `Commands::write_message`

## ECS and System Parameters

### Generic Option Parameter

`Option<Single<D, F>>` now resolves to `None` if there are **multiple** entities matching the query. In 0.16, it would
skip the system if there were multiple.

- If you need to handle multiple entities or skip, use `Query<D, F>` and check `single()` or `iter()`.

### Renamed Condition to SystemCondition

The `Condition` trait used in `.run_if()` is now `SystemCondition`. This usually only affects imports if you are
implementing custom conditions.

### Internal Entities

Observers and registered one-shot systems are now marked with the `Internal` component. They are hidden from most
queries by default. To query them, you must add the `Allow<Internal>` filter.

### EntityRef and EntityMut

`EntityRef` and `EntityMut` no longer ignore default query filters (like `Disabled`). If you need to include disabled
entities, use `Allows<Disabled>`.

## Picking and UI

### Pointer Event Renames

- `Pointer<Pressed>` -> `Pointer<Press>`
- `Pointer<Released>` -> `Pointer<Release>`
- `Pointer<Over>`, `Pointer<Out>`, `Pointer<Click>` remain but are often used via `MessageReader`.

### RelativeCursorPosition

Coordinates are now **object-centered**. (0,0) is the center of the node, and corners are at (±0.5, ±0.5).

### Text2d

`Text2d` has moved to `bevy_sprite`. In Unhaunter, check imports in systems rendering world-space text.

## Assets and Rendering

### Handle::Uuid

`Handle::Weak` is replaced by `Handle::Uuid`. Use `uuid_handle!` macro for static handles.

### Hdr Component

`Camera.hdr: bool` is now a separate component `Hdr`.

```rust
// 0.17+
commands.spawn((Camera3d, GCameraArena, Hdr));
```

### Assets::insert

`Assets::insert` and `Assets::get_or_insert_with` now return a `Result`. If the handle was dropped, it returns an error
instead of panicking.

## Core Utilities

### Timer Methods

- `timer.paused()` -> `timer.is_paused()`
- `timer.finished()` -> `timer.is_finished()`

### SceneSpawner

Methods now distinguish between `Scene` and `DynamicScene`. Use `despawn_dynamic` for `DynamicScene` handles.
