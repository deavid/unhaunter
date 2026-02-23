# Bevy Migration Reference for Unhaunter (0.18+)

This document filters the Bevy 0.18 migration guide for AI agents working on Unhaunter.

## ECS and Scheduling

### Removed SimpleExecutor

`SimpleExecutor` has been removed. The project explicitly sets its executors to `SingleThreadedExecutor` (via
`ExecutorKind::SingleThreaded`) for dedicated servers to reduce overhead.

- **AI Tactic**: Ensure system ordering is explicit using `.before()`, `.after()`, or `.chain()` rather than relying on
  the order of `add_systems`.

### Inherited Entity Events

`EntityEvent` is now immutable. If you need to set the event target, use the `SetEntityEventTarget` trait (usually
handled automatically by propagation).

### System Combinators

Logic operators on system conditions (e.g., `condition.and(other)`) no longer propagate validation errors. If a system
in a combinator fails validation (e.g., a `Single` query matches zero entities), it is treated as `false` rather than an
error/skip.

## Component Changes

### Node and BorderRadius

`BorderRadius` is **no longer a component**. It is now a field on the `Node` component.

```rust
// 0.18+
Node {
    border_radius: BorderRadius::all(Val::Px(10.0)),
    ..default()
}
```

### RenderTarget

`RenderTarget` is **now a component**, not a field on `Camera`.

```rust
// 0.18+
commands.spawn((
    Camera2d::default(),
    RenderTarget::Image(image_handle),
    MCamera, // Marker component
));
```

## Assets

### Mesh Access

`Mesh` operations like `insert_attribute` now have `try_*` variants (e.g., `try_insert_attribute`) for meshes that might
have been extracted to the render world.

- **Use**: Wrap mesh modification in `.try_...().expect(...)` or handle `Result`.

### AssetPath in Loaders

`LoadContext::path()` now returns an `AssetPath` instead of a raw `&Path`.

- To get the source path as a legacy `&Path`, use `load_context.path().path()`.
- **Note**: Prefer `AssetPath` for better support of custom asset sources.

### Winit User Events

`WinitUserEvent` is used for waking up the event loop (e.g., `WinitUserEvent::WakeUp`).

## Reflection

`#[reflect(...)]` attributes now **only support parentheses**. `#[reflect[Clone]]` or `#[reflect{Clone}]` are invalid.

- **AI Tactic**: Always use `#[reflect(Component, Default)]` etc.

## Change Detection

Change detection types (e.g., `Tick`, `ComponentTicks`) have moved from `bevy::ecs::component` to
`bevy::ecs::change_detection`.
