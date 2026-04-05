use bevy::prelude::*;
use uncommon_states_core::BootState;
use untmxmap_core::resources::maps::Maps;

pub(crate) fn set_boot_ready_when_maps_loaded(
    maps: Option<Res<Maps>>,
    boot_state: Res<State<BootState>>,
    mut next_boot: ResMut<NextState<BootState>>,
) {
    if *boot_state == BootState::Ready {
        return;
    } // one-way gate
    // FIXME: This is incorrect. We can get a partial map list and it's not fully loaded.
    if let Some(m) = maps.as_ref().filter(|m| !m.maps.is_empty()) {
        info!("BootState -> Ready ({} maps loaded)", m.maps.len());
        next_boot.set(BootState::Ready);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, set_boot_ready_when_maps_loaded);
}
