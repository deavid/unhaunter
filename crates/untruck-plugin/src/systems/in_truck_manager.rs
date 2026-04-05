use bevy::prelude::*;
use unplayer_core::components::Hiding;
use untruck_core::components::in_truck::InTruck;

fn on_intruck_added(trigger: On<Add, InTruck>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(Hiding { hiding_spot: None });
}

fn on_intruck_removed(trigger: On<Remove, InTruck>, mut commands: Commands) {
    commands.entity(trigger.entity).remove::<Hiding>();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_observer(on_intruck_added);
    app.add_observer(on_intruck_removed);
}
