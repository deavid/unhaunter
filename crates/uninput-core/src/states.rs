use bevy::prelude::*;

/// Tracks which UI modal (if any) is currently open during an active mission.
///
/// This is the single authoritative switch for modal exclusivity in-game.
/// Only one variant can be active at a time. Owned by `uninput-core` because input
/// focus is the primary consumer: `InGameUiState::Running` ↔ `MissionInputFocus::has_focus = true`.
#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InGameUiState {
    /// No modal is open. The player has full input focus.
    #[default]
    Running,
    /// The truck loadout / journal UI is open.
    Truck,
    /// The pause menu is open.
    Pause,
    /// An NPC help dialog is open.
    NpcHelp,
}
