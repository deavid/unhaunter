use crate::colors;
use crate::platform::plt::FONT_SCALE;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents the visual state of a tab in the truck UI.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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
#[derive(Debug, Clone, Component, PartialEq, Eq, Serialize, Deserialize)]
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

impl TruckTab {
    /// Updates the tab's visual state based on the given interaction.
    pub fn update_from_interaction(&mut self, interaction: &Interaction) {
        match self.state {
            TabState::Disabled | TabState::Selected => {}
            TabState::Default | TabState::Hover | TabState::Pressed => {
                self.state = match interaction {
                    Interaction::Pressed => TabState::Pressed,
                    Interaction::Hovered => TabState::Hover,
                    Interaction::None => TabState::Default,
                };
            }
        }
    }

    pub fn text_color(&self) -> Color {
        match self.state {
            TabState::Selected => colors::TRUCKUI_BGCOLOR.with_alpha(1.0),
            TabState::Pressed => colors::TRUCKUI_BGCOLOR.with_alpha(0.8),
            TabState::Hover => colors::TRUCKUI_ACCENT2_COLOR.with_alpha(0.6),
            TabState::Default => Hsla::from(colors::TRUCKUI_ACCENT_COLOR)
                .with_saturation(0.1)
                .with_alpha(0.6)
                .into(),
            TabState::Disabled => colors::INVENTORY_STATS_COLOR.with_alpha(0.05),
        }
    }

    pub fn bg_color(&self) -> Color {
        match self.state {
            TabState::Pressed => colors::TRUCKUI_ACCENT2_COLOR,
            TabState::Selected => colors::TRUCKUI_ACCENT_COLOR,
            TabState::Hover => colors::TRUCKUI_BGCOLOR,
            TabState::Default => colors::TRUCKUI_BGCOLOR.with_alpha(0.7),
            TabState::Disabled => colors::TRUCKUI_BGCOLOR.with_alpha(0.5),
        }
    }

    pub fn font_size(&self) -> f32 {
        match self.state {
            TabState::Selected => 35.0 * FONT_SCALE,
            _ => 24.0 * FONT_SCALE,
        }
    }
}
