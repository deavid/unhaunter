use bevy::prelude::*;
use unbehavior::behavior::{Behavior, Interactive, NpcHelpDialog};
use unbehavior::components;
use unmapload_core::components::PendingTiledLayerProperties;
use untypes_core::hydration::HydrationStage;

fn hydration_npc_system(
    mut q: Query<(Entity, &Behavior, &PendingTiledLayerProperties), With<HydrationStage<3>>>,
    mut commands: Commands,
) {
    use bevy::picking::Pickable;
    for (entity, behavior, pending_props) in q.iter_mut() {
        if behavior.p.is_npc {
            let cfg = behavior.cfg();
            commands
                .entity(entity)
                .insert(Pickable::default())
                .insert(NpcHelpDialog::new("NPC", &cfg.variant, &pending_props.0))
                .insert(Interactive::new(
                    "sounds/effects-dongdongdong.ogg",
                    "sounds/effects-dongdongdong.ogg",
                ))
                .insert(components::FloorItemCollidable);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, hydration_npc_system);
}
