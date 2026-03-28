use bevy::prelude::*;

use crate::types::tab::{TabContents, TabState};

/// Represents a tab in the truck UI.
#[derive(Debug, Clone, Component)]
pub struct TruckTab {
    /// The display name of the tab.
    pub tabname: String,
    /// The current visual state of the tab.
    pub state: TabState,
    /// The content section associated with the tab.
    pub contents: TabContents,
}
