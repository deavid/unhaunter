# Custom Sprite Picking Backend

This module provides a custom picking backend for map sprites that use `MeshMaterial2d<CustomMaterial1>` instead of the standard `Sprite` component.

## Usage

Add the plugin to your app:

```rust
use unstd::picking::{CustomSpritePickingPlugin, CustomSpritePickingSettings};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_picking::DefaultPickingPlugins)
        .add_plugins(CustomSpritePickingPlugin)
        .insert_resource(CustomSpritePickingSettings::new())
        // ... rest of your app
        .run();
}
```

## Required Components

For entities to be pickable, they need:
- `MeshMaterial2d<CustomMaterial1>` - Your custom material
- `GlobalTransform` - World position and transform
- `Pickable` - Picking configuration (use `Pickable::default()` or `Pickable::IGNORE`)
- `ViewVisibility` - Visibility status

## Settings

Configure the picking behavior with `CustomSpritePickingSettings`:

```rust
// Default settings (1x1 logical unit bounding box)
let settings = CustomSpritePickingSettings::new();

// Custom tile size
let settings = CustomSpritePickingSettings {
    tile_size: Vec2::new(1.0, 1.0), // Adjust based on your sprites
    picking_mode: CustomSpritePickingMode::BoundingBox,
    require_markers: false,
};
```

## Camera Markers

If you want fine-grained control over which cameras can pick:

```rust
// Enable marker requirement
settings.require_markers = true;

// Mark specific cameras for picking
commands.spawn((
    Camera2dBundle::default(),
    CustomSpritePickingCamera, // Only this camera will perform picking
));
```

## Integration with Mouse Interaction

This picking backend works seamlessly with the existing mouse interaction system. Once an entity is picked, it will generate the appropriate `Pointer<Click>` events that your `mouse_interaction_system` can handle.
