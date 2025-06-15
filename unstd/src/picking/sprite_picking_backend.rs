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

/// Alpha threshold for pixel-perfect picking (80%)
const ALPHA_THRESHOLD: f32 = 0.8;

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
    /// This mode samples the actual texture at the click point and checks if the alpha
    /// value is above the threshold (80%). This provides accurate hit detection that
    /// respects sprite transparency and actual sprite bounds.
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
            picking_mode: CustomSpritePickingMode::AlphaThreshold(ALPHA_THRESHOLD),
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
    materials: Res<Assets<CustomMaterial1>>,
    images: Res<Assets<Image>>,
    mut output: EventWriter<PointerHits>,
) {
    // Filter and sort sprites by depth (Z order) for proper picking priority
    let mut sorted_sprites: Vec<_> = sprite_query
        .iter()
        .filter_map(|(entity, material_handle, transform, pickable, vis)| {
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

            Some((entity, material_handle, transform, pickable))
        })
        .collect();

    // Sort by depth (front to back for proper picking order)
    // Higher Z values are closer to the camera and should be picked first
    sorted_sprites.sort_by(|(_, _, transform_a, _), (_, _, transform_b, _)| {
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
            .filter_map(|(entity, material_handle, sprite_transform, pickable)| {
                if blocked {
                    return None;
                }

                // Project cursor ray onto the sprite's Z-plane
                let t = (sprite_transform.translation().z - cursor_ray_world.origin.z)
                    / cursor_ray_world.direction.z;
                let cursor_at_sprite_z = cursor_ray_world.origin + *cursor_ray_world.direction * t;
                let cursor_2d = cursor_at_sprite_z.truncate();

                // Perform hit testing based on picking mode
                let hit = match settings.picking_mode {
                    CustomSpritePickingMode::BoundingBox => {
                        // Simple bounding box intersection using tile_size
                        let sprite_center = sprite_transform.translation().truncate();
                        let half_tile = settings.tile_size * 0.5;
                        let min_bounds = sprite_center - half_tile;
                        let max_bounds = sprite_center + half_tile;

                        cursor_2d.x >= min_bounds.x
                            && cursor_2d.x <= max_bounds.x
                            && cursor_2d.y >= min_bounds.y
                            && cursor_2d.y <= max_bounds.y
                    }
                    CustomSpritePickingMode::AlphaThreshold(_) => {
                        // TODO: Add caching if performance becomes an issue
                        pixel_perfect_hit_test(
                            cursor_2d,
                            sprite_transform,
                            material_handle,
                            &materials,
                            &images,
                        )
                    }
                };

                if !hit {
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

/// Performs pixel-perfect hit testing for a sprite using texture sampling.
///
/// This function:
/// 1. Extracts sprite dimensions and sheet layout from the material
/// 2. Calculates the proper bounding box using sprite anchoring
/// 3. Converts world coordinates to sprite-local UV coordinates
/// 4. Samples the texture at the calculated UV position
/// 5. Checks if the alpha value exceeds the threshold
///
/// Returns `true` if the click hit an opaque part of the sprite, `false` otherwise.
fn pixel_perfect_hit_test(
    cursor_world_pos: Vec2,
    sprite_transform: &GlobalTransform,
    material_handle: &MeshMaterial2d<CustomMaterial1>,
    materials: &Assets<CustomMaterial1>,
    images: &Assets<Image>,
) -> bool {
    // Get the material data
    let Some(material) = materials.get(&material_handle.0) else {
        warn!("CustomSpritePicking: Material not found for pixel-perfect picking");
        return false;
    };

    // Get the texture image
    let Some(image) = images.get(material.texture()) else {
        // Image not loaded yet, treat as non-pickable
        return false;
    };

    // Extract sprite dimensions from material
    let sprite_size = Vec2::new(material.data.sprite_width, material.data.sprite_height);

    // Calculate sprite anchor point using the y_anchor from the material
    let y_anchor = material.data.y_anchor;
    let sprite_anchor = Vec2::new(sprite_size.x / 2.0, sprite_size.y * (0.5 - y_anchor));

    // Calculate sprite bounds in world space (similar to QuadCC logic)
    let sprite_center = sprite_transform.translation().truncate();
    let left_x = sprite_center.x - sprite_anchor.x;
    let right_x = sprite_center.x + (sprite_size.x - sprite_anchor.x);
    let bottom_y = sprite_center.y + (sprite_anchor.y - sprite_size.y);
    let top_y = sprite_center.y + sprite_anchor.y;

    // Check if cursor is within sprite bounds
    if cursor_world_pos.x < left_x
        || cursor_world_pos.x > right_x
        || cursor_world_pos.y < bottom_y
        || cursor_world_pos.y > top_y
    {
        return false;
    }

    // Convert world position to local sprite coordinates (0.0 to 1.0)
    let local_x = (cursor_world_pos.x - left_x) / sprite_size.x;
    let local_y = (cursor_world_pos.y - bottom_y) / sprite_size.y;

    // Handle horizontal flipping - check if sprite is flipped
    let is_flipped = sprite_transform.affine().x_axis.x < 0.0;
    let (u_local, v_local) = if is_flipped {
        (1.0 - local_x, local_y)
    } else {
        (local_x, local_y)
    };

    // Calculate UV coordinates within the sprite sheet
    let sheet_cols = material.data.sheet_cols as f32;
    let sheet_rows = material.data.sheet_rows as f32;
    let sheet_idx = material.data.sheet_idx as f32;

    let col = sheet_idx % sheet_cols;
    let row = (sheet_idx / sheet_cols).floor();

    let cell_width = 1.0 / sheet_cols;
    let cell_height = 1.0 / sheet_rows;

    let base_u = col * cell_width;
    let base_v = row * cell_height;

    // Apply margin protection like the shader does
    let margin = 0.5;
    let mx = margin / material.data.sprite_width;
    let my = margin / material.data.sprite_height;

    let margin_u = u_local.clamp(0.0, 1.0 - mx);
    let margin_v = v_local.clamp(my * 2.0, 1.0 - my);

    let final_u = base_u + margin_u * cell_width;
    let final_v = base_v + margin_v * cell_height;

    // Sample the texture at the calculated UV coordinates
    let tex_x = (final_u * image.width() as f32) as u32;
    let tex_y = (final_v * image.height() as f32) as u32;

    // Clamp to texture bounds
    let tex_x = tex_x.min(image.width() - 1);
    let tex_y = tex_y.min(image.height() - 1);

    // Sample the pixel - calculate the pixel index in the raw data
    let pixel_index = (tex_y * image.width() + tex_x) as usize;

    // Get pixel data based on texture format
    match &image.texture_descriptor.format {
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb
        | bevy::render::render_resource::TextureFormat::Rgba8Unorm => {
            if let Some(ref pixel_data) = image.data {
                let alpha_index = pixel_index * 4 + 3;
                if alpha_index < pixel_data.len() {
                    let alpha_byte = pixel_data[alpha_index];
                    let alpha = alpha_byte as f32 / 255.0;
                    alpha > ALPHA_THRESHOLD
                } else {
                    warn!("CustomSpritePicking: Pixel index out of bounds");
                    false
                }
            } else {
                warn!("CustomSpritePicking: Image data not available");
                false
            }
        }
        _ => {
            // For other formats, assume opaque for now
            // TODO: Add support for more texture formats if needed
            true
        }
    }
}
