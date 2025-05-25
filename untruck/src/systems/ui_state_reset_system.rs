use bevy::prelude::*;
use uncore::states::GameState;
use unwalkiecore::resources::WalkiePlay;

fn reset_craft_button_highlight_on_truck_exit_system(
    mut prev_game_state: Local<GameState>,
    current_game_state: Res<State<GameState>>,
    mut walkie_play: ResMut<WalkiePlay>,
) {
    let current_gs_val = *current_game_state.get();
    let previous_gs_val = *prev_game_state;

    // Initialize prev_game_state on first run
    if !previous_gs_val.eq(&current_gs_val) && previous_gs_val == GameState::default() {
        // This check might be needed if Local defaults to something other than what State might initially be
        // Or, more simply, just update prev_game_state at the end.
    }

    if current_gs_val == GameState::None
        && previous_gs_val == GameState::Truck
        && walkie_play.highlight_craft_button
    {
        walkie_play.highlight_craft_button = false;
        // info!("Resetting craft button highlight on truck exit (GameState::None from GameState::Truck).");
    }

    // Update prev_game_state for the next frame
    *prev_game_state = current_gs_val;
}

// This system could also be added to the main systems.rs if preferred,
// but a dedicated file for UI state resets can be cleaner.
// For now, creating it here as specified by the plan.
// Need to ensure this new module is declared in untruck/src/lib.rs or untruck/src/systems.rs
// e.g., in untruck/src/systems.rs:
// pub mod ui_state_reset_systems;
// And in untruck/src/lib.rs (if systems.rs is not pub using it):
// mod systems;
// pub use systems::ui_state_reset_systems; // or similar path
// Or directly declare in untruck/src/plugin.rs if that's the pattern.
// For this task, I'll assume it's added to a new `systems` submodule.
// The plugin modification will reflect this.
// If untruck/src/systems.rs is the main aggregator:
// In untruck/src/systems.rs add: `pub mod ui_state_reset_systems;`
// Then in untruck/src/plugin.rs use `systems::ui_state_reset_systems::reset_craft_button_highlight_on_truck_exit_system`
// Or if ui_state_reset_systems is directly under untruck/src:
// In untruck/src/lib.rs add: `pub mod ui_state_reset_systems;`
// Then in untruck/src/plugin.rs use `crate::ui_state_reset_systems::reset_craft_button_highlight_on_truck_exit_system`

// Simpler: Assume direct use for now, adjust if plugin shows different structure.
// The `ls` for untruck/src showed a `systems.rs` and a `systems/` directory.
// I'll assume `untruck/src/systems.rs` can declare `pub mod ui_state_reset_systems;`
// and `untruck/src/systems/ui_state_reset_systems.rs` is the file.
// Let's check untruck/src/systems.rs to see how modules are handled.
// For now, I'll proceed with creating the file as above.
// The plugin step will clarify the path.
// Actually, the prompt says: "in untruck/src/systems/ui_state_reset_systems.rs or a similar new file"
// This implies it should be a new file within the systems directory.
// So the path would be `untruck/src/systems/ui_state_reset_systems.rs`
// And in `untruck/src/systems/mod.rs` (or `untruck/src/systems.rs` if it's a file acting as mod.rs):
// `pub mod ui_state_reset_systems;`
// Then in plugin: `use crate::systems::ui_state_reset_systems::reset_craft_button_highlight_on_truck_exit_system;`
// This seems like the most standard Rust module structure.
// The Local<GameState> initialization will be handled by Bevy correctly. The first time the system runs,
// prev_game_state will be the default for GameState. The current_game_state will be the actual current state.
// The comparison `previous_gs_val == GameState::Truck` and `current_gs_val == GameState::None`
// will only be true after at least one frame where previous_gs_val became GameState::Truck.
// So, the initial default value of prev_game_state (e.g. GameState::None or GameState::Menu)
// won't cause a false trigger on the first frame the game enters GameState::None from an initial state.
// The logic `*prev_game_state = current_gs_val;` at the end is key.

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, reset_craft_button_highlight_on_truck_exit_system);
}
