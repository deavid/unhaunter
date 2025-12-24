use bevy::prelude::*;
use uncore_events::events::loadlevel::LevelLoadedEvent;
use ungear::resources::spawner::GearSpawnerRegistry;

#[derive(Debug, Resource, Clone, Default)]
pub struct TruckGear {
    pub inventory: Vec<Entity>,
}

pub fn app_setup(app: &mut App) {
    app.init_resource::<TruckGear>();
    app.add_systems(Update, initialize_truck_gear);
}

fn initialize_truck_gear(
    mut ev_level: MessageReader<LevelLoadedEvent>,
    mut truck_gear: ResMut<TruckGear>,
    difficulty: Res<undifficulty::CurrentDifficulty>,
    gear_registry: Res<GearSpawnerRegistry>,
    mut commands: Commands,
) {
    for _ in ev_level.read() {
        // Despawn old gear if any
        for entity in truck_gear.inventory.drain(..) {
            commands.entity(entity).despawn();
        }

        for kind in &difficulty.0.truck_gear {
            let entity = gear_registry.spawn(&mut commands, *kind);
            truck_gear.inventory.push(entity);
        }
    }
}
