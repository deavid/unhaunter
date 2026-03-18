use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents the visual state of a tab in the truck UI.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum TabState {
    /// The tab is currently selected and active.
    Selected,
    /// The tab is being pressed.
    Pressed,
    /// The mouse is hovering over the tab.
    Hover,
    /// The tab is in its default, unselected state.
    #[default]
    Default,
    /// The tab is disabled and cannot be interacted with.
    Disabled,
}

/// Represents the different content sections within the truck UI.
#[derive(Debug, Clone, Component, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum TabContents {
    /// The loadout tab for managing player gear.
    Loadout,
    /// The location map tab. (Currently disabled)
    LocationMap,
    /// The camera feed tab. (Currently disabled)
    CameraFeed,
    /// The journal tab for reviewing evidence and guessing the ghost type.
    Journal,
}

impl TabContents {
    /// Returns the display name for the tab content.
    pub fn name(&self) -> &'static str {
        match self {
            TabContents::Loadout => "Loadout",
            TabContents::LocationMap => "Location Map",
            TabContents::CameraFeed => "Camera Feed",
            TabContents::Journal => "Journal",
        }
    }

    /// Returns the default `TabState` for the tab content.
    pub fn default_state(&self) -> TabState {
        match self {
            TabContents::Loadout => TabState::Default,
            TabContents::LocationMap => TabState::Disabled,
            TabContents::CameraFeed => TabState::Disabled,
            TabContents::Journal => TabState::Default,
        }
    }
}
