use bevy::prelude::*;
use ungear_core::difficulty_ext::DifficultyGearExt;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use unmapload_core::events::loadlevel::LevelLoadedEvent;
use untruck_core::truckgear::TruckGear;

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<TruckGear>();
    app.add_systems(
        Update,
        initialize_truck_gear.run_if(resource_exists::<untypes_core::roles::AuthorityRole>),
    );
}

fn initialize_truck_gear(
    mut ev_level: MessageReader<LevelLoadedEvent>,
    mut truck_gear: ResMut<TruckGear>,
    difficulty: Res<undifficulty_core::current_difficulty::CurrentDifficulty>,
    gear_registry: Res<GearSpawnerRegistry>,
    mut commands: Commands,
) {
    for _ in ev_level.read() {
        // Despawn old gear if any
        for entity in truck_gear.inventory.drain(..) {
            commands.entity(entity).despawn();
        }

        for kind in &difficulty.0.truck_gear() {
            let entity = gear_registry.spawn(&mut commands, *kind);
            truck_gear.inventory.push(entity);
        }
    }
}
