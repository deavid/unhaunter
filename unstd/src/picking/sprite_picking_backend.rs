//! Custom sprite picking backend for map sprites using custom materials
//!
//! This backend enables mouse picking for map sprites that use `MeshMaterial2d<CustomMaterial1>`
//! instead of the standard `Sprite` component. It mimics the behavior of Bevy's built-in
//! `SpritePickingPlugin` but works with custom materials and mesh-based sprites.
//!
//! ## Architecture
//!
//! The picking system works by:
//! 1. Filtering visible sprites with the required components
//! 2. Sorting them by depth (Z-order) for proper picking priority
//! 3. Converting screen coordinates to world space using camera projection
//! 4. Testing bounding box intersections for each sprite
//! 5. Emitting `PointerHits` events for successful picks
//!
//! ## Performance
//!
//! This implementation uses simple bounding box collision detection for performance.
//! For pixel-perfect picking, the `CustomSpritePickingMode::AlphaThreshold` variant
//! is provided but not yet implemented.

use crate::materials::CustomMaterial1;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_picking::backend::prelude::*;

/// An optional component that marks cameras that should be used for custom sprite picking.
///
/// Only needed if [`CustomSpritePickingSettings::require_markers`] is set to `true`.
#[derive(Debug, Clone, Default, Component)]
pub struct CustomSpritePickingCamera;

/// How the custom sprite picking backend should handle sprite boundaries.
#[derive(Debug, Clone, Copy)]
pub enum CustomSpritePickingMode {
    /// Only consider the logical bounding box of sprites.
    ///
    /// This is the fastest method and works well for most use cases.
    /// The bounding box size is determined by [`CustomSpritePickingSettings::tile_size`].
    BoundingBox,
    /// Use pixel-perfect picking with transparency threshold.
    ///
    /// **Note:** This mode is not yet implemented and will fall back to `BoundingBox`.
    /// When implemented, it will require access to texture data for accurate transparency testing.
    AlphaThreshold(f32),
}

impl Default for CustomSpritePickingMode {
    fn default() -> Self {
        Self::BoundingBox
    }
}

/// Runtime settings for the custom sprite picking backend.
#[derive(Resource)]
pub struct CustomSpritePickingSettings {
    /// When `true`, only cameras marked with [`CustomSpritePickingCamera`] will perform picking.
    ///
    /// When `false` (default), all active cameras will be considered for picking.
    /// This is useful for fine-grained control in multi-camera setups.
    pub require_markers: bool,

    /// Determines how sprite boundaries are calculated for hit testing.
    pub picking_mode: CustomSpritePickingMode,

    /// Size of the clickable area around each sprite in world units.
    ///
    /// This defines the bounding box dimensions for hit testing.
    /// Default is 16x16 pixels, which works well for typical sprite sizes.
    /// Adjust based on your sprite dimensions and desired click tolerance.
    pub tile_size: Vec2,
}

impl Default for CustomSpritePickingSettings {
    fn default() -> Self {
        Self {
            require_markers: false,
            picking_mode: CustomSpritePickingMode::BoundingBox,
            tile_size: Vec2::new(16.0, 16.0),
        }
    }
}

/// Plugin that enables custom sprite picking for map sprites
#[derive(Clone)]
pub struct CustomSpritePickingPlugin;

impl Plugin for CustomSpritePickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CustomSpritePickingSettings>()
            .add_systems(PreUpdate, custom_sprite_picking.in_set(PickSet::Backend));
    }
}

/// Main picking system that handles mouse interaction with custom map sprites.
///
/// This system runs in the `PickSet::Backend` schedule and processes pointer events
/// to determine which sprites are being clicked. It emits `PointerHits` events that
/// can be consumed by other systems.
///
/// ## Process
///
/// 1. **Sprite Filtering**: Finds all visible, pickable sprites with custom materials
/// 2. **Depth Sorting**: Orders sprites by Z-coordinate (front to back)
/// 3. **Camera Matching**: Locates the appropriate camera for each pointer
/// 4. **World Projection**: Converts screen coordinates to world space
/// 5. **Hit Testing**: Checks bounding box intersections
/// 6. **Event Emission**: Sends `PointerHits` for successful picks
fn custom_sprite_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(
        Entity,
        &Camera,
        &GlobalTransform,
        &Projection,
        Has<CustomSpritePickingCamera>,
    )>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    settings: Res<CustomSpritePickingSettings>,
    sprite_query: Query<(
        Entity,
        &MeshMaterial2d<CustomMaterial1>,
        &GlobalTransform,
        &Pickable,
        &ViewVisibility,
    )>,
    mut output: EventWriter<PointerHits>,
) {
    // Filter and sort sprites by depth (Z order) for proper picking priority
    let mut sorted_sprites: Vec<_> = sprite_query
        .iter()
        .filter_map(|(entity, _material, transform, pickable, vis)| {
            // Only include visible sprites
            if !vis.get() {
                return None;
            }

            // Check if picking is enabled for this entity
            if pickable == &Pickable::IGNORE {
                return None;
            }

            // Ensure transform is valid (no NaN or infinite values)
            if !transform.affine().is_finite() {
                return None;
            }

            Some((entity, transform, pickable))
        })
        .collect();

    // Sort by depth (front to back for proper picking order)
    // Higher Z values are closer to the camera and should be picked first
    sorted_sprites.sort_by(|(_, transform_a, _), (_, transform_b, _)| {
        transform_b
            .translation()
            .z
            .partial_cmp(&transform_a.translation().z)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let primary_window = primary_window.single().ok();

    // Process each pointer (mouse, touch, etc.)
    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let mut blocked = false;

        // Find the appropriate camera for this pointer's target window
        let Some((
            cam_entity,
            camera,
            cam_transform,
            Projection::Orthographic(cam_ortho),
            _cam_can_pick,
        )) = cameras
            .iter()
            .filter(|(_, camera, _, _, cam_can_pick)| {
                let marker_requirement = !settings.require_markers || *cam_can_pick;
                camera.is_active && marker_requirement
            })
            .find(|(_, camera, _, _, _)| {
                camera
                    .target
                    .normalize(primary_window)
                    .is_some_and(|x| x == location.target)
            })
        else {
            continue;
        };

        let viewport_pos = camera
            .logical_viewport_rect()
            .map(|v| v.min)
            .unwrap_or_default();
        let pos_in_viewport = location.position - viewport_pos;

        // Convert screen position to world ray for 3D intersection
        let Ok(cursor_ray_world) = camera.viewport_to_world(cam_transform, pos_in_viewport) else {
            continue; // Skip if viewport-to-world conversion fails
        };

        let picks: Vec<(Entity, HitData)> = sorted_sprites
            .iter()
            .copied()
            .filter_map(|(entity, sprite_transform, pickable)| {
                if blocked {
                    return None;
                }

                // Simple bounding box intersection - similar to original but using tile_size
                let sprite_center = sprite_transform.translation().truncate();
                let half_tile = settings.tile_size * 0.5;

                // Create axis-aligned bounding box around the sprite
                let min_bounds = sprite_center - half_tile;
                let max_bounds = sprite_center + half_tile;

                // Project cursor ray onto the sprite's Z-plane
                let t = (sprite_transform.translation().z - cursor_ray_world.origin.z)
                    / cursor_ray_world.direction.z;
                let cursor_at_sprite_z = cursor_ray_world.origin + cursor_ray_world.direction * t;
                let cursor_2d = cursor_at_sprite_z.truncate();

                // Check if cursor is within bounds
                let cursor_in_bounds = cursor_2d.x >= min_bounds.x
                    && cursor_2d.x <= max_bounds.x
                    && cursor_2d.y >= min_bounds.y
                    && cursor_2d.y <= max_bounds.y;

                // Log the hit detection math for debugging
                if cursor_in_bounds {
                    // info!(
                    //     "CustomSpritePicking: HIT! Entity {:?} - sprite_center={:?}, bounds=[{:?}, {:?}], cursor_2d={:?}",
                    //     entity, sprite_center, min_bounds, max_bounds, cursor_2d
                    // );
                }

                if !cursor_in_bounds {
                    return None;
                }

                blocked = pickable.should_block_lower;

                // Calculate hit data - similar to original SpritePickingPlugin
                let hit_pos_world = cursor_at_sprite_z;

                // Calculate depth from camera
                let hit_pos_cam = cam_transform
                    .affine()
                    .inverse()
                    .transform_point3(hit_pos_world);
                let depth = -cam_ortho.near - hit_pos_cam.z;

                Some((
                    entity,
                    HitData::new(
                        cam_entity,
                        depth,
                        Some(hit_pos_world),
                        Some(*sprite_transform.back()),
                    ),
                ))
            })
            .collect();

        let order = camera.order as f32;
        if !picks.is_empty() {
            // info!(
            //     "CustomSpritePicking: Writing {} picks for pointer {:?} with order {}",
            //     picks.len(),
            //     pointer,
            //     order
            // );
            for (entity, hit_data) in &picks {
                debug!(
                    "CustomSpritePicking: Pick hit entity {:?} at depth {:.2}",
                    entity, hit_data.depth
                );
            }
            output.write(PointerHits::new(*pointer, picks, order));
        }
    }
}
