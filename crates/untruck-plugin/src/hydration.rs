use bevy::prelude::*;
use unbehavior_core::behavior::{Behavior, Interactive};
use unbehavior_core::components;
use untypes_core::hydration::HydrationStage;

fn hydration_van_entry_system(
    mut q: Query<(Entity, &Behavior), With<HydrationStage<3>>>,
    mut commands: Commands,
) {
    use bevy::picking::Pickable;
    for (entity, behavior) in q.iter_mut() {
        if behavior.p.is_van_entry {
            commands
                .entity(entity)
                .insert(Pickable::default())
                .insert(Interactive::new(
                    "sounds/door-open.ogg",
                    "sounds/door-close.ogg",
                ))
                .insert(components::FloorItemCollidable);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, hydration_van_entry_system);
}
