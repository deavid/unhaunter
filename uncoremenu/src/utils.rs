use bevy::prelude::*;

/// Converts an index to a z-index value, ensuring proper layering
pub fn layer_to_z_index(layer: i32) -> ZIndex {
    ZIndex(layer)
}

/// Helper function to determine if a position is within an entity's bounds
pub fn is_position_inside_node(position: Vec2, node_size: Vec2, node_position: Vec2) -> bool {
    position.x >= node_position.x
        && position.x <= node_position.x + node_size.x
        && position.y >= node_position.y
        && position.y <= node_position.y + node_size.y
}

/// Helper function to create a standard menu selection event
pub fn create_menu_selection_system<
    T: Component + Copy + Send + Sync + 'static,
    E: Event + From<T>,
>() -> impl FnMut(Query<&T, With<Interaction>>, EventWriter<E>) + Send + Sync + 'static {
    move |query: Query<&T, With<Interaction>>, mut event_writer: EventWriter<E>| {
        for menu_identifier in query.iter() {
            event_writer.send(E::from(*menu_identifier));
        }
    }
}
