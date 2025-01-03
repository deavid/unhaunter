use crate::components::truck_ui_button::TruckUIButton;
use crate::types::{evidence::Evidence, ghost::types::GhostType};

/// Represents the type of a button in the truck UI.
#[derive(Debug, PartialEq, Eq)]
pub enum TruckButtonType {
    /// A button for selecting or discarding a piece of evidence.
    Evidence(Evidence),
    /// A button for selecting or discarding a ghost type guess.
    Ghost(GhostType),
    /// The button for crafting a ghost repellent.
    CraftRepellent,
    /// The button for exiting the truck.
    ExitTruck,
    /// The button for ending the current mission.
    EndMission,
}

impl TruckButtonType {
    /// Creates a `TruckUIButton` component from a `TruckButtonType`.
    pub fn into_component(self) -> TruckUIButton {
        TruckUIButton::from(self)
    }
}

/// Represents the state of a button in the truck UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruckButtonState {
    /// The button is in its default, unselected state.
    Off,
    /// The button is selected or pressed.
    Pressed,
    /// The button is in a discarded state (e.g., for evidence or ghost guesses).
    Discard,
}
