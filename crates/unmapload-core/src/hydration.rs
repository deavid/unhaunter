use bevy::ecs::component::Component;

/// Marker component for the staged map hydration pipeline.
///
/// Each tile entity begins at `HydrationStage<1>` after spawning and is advanced
/// through stages by the conveyor system in `unmapload-plugin`. Domain plugins attach
/// their components during the appropriate stage and the conveyor removes the marker
/// when all stages are complete.
#[derive(Component)]
pub struct HydrationStage<const S: usize>;
