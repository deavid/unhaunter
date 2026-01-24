use bevy::ecs::component::Component;
use bevy_platform::collections::HashMap;

/// Temporary component to carry over Tiled layer properties during the hydration pipeline.
#[derive(Component)]
pub struct PendingTiledLayerProperties(pub HashMap<String, tiled::PropertyValue>);
