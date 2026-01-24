use bevy::ecs::component::Component;

/// Marker component for the staged map hydration pipeline.
#[derive(Component)]
pub struct HydrationStage<const S: usize>;
