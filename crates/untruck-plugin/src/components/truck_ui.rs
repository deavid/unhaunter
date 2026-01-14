use bevy::prelude::*;
use unfoundation_core::colors;
use unfoundation_core::platform::plt::FONT_SCALE;
pub(crate) use unfoundation_core::types::truck::{TabContents, TabState};

/// Represents a tab in the truck UI.
#[derive(Debug, Clone, Component)]
pub(crate) struct TruckTab {
    /// The display name of the tab.
    pub tabname: String,
    /// The current visual state of the tab.
    pub state: TabState,
    /// The content section associated with the tab.
    pub contents: TabContents,
}

impl TruckTab {
    /// Updates the tab's visual state based on the given interaction.
    pub(crate) fn update_from_interaction(&mut self, interaction: &Interaction) {
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

    pub(crate) fn text_color(&self) -> Color {
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

    pub(crate) fn bg_color(&self) -> Color {
        match self.state {
            TabState::Pressed => colors::TRUCKUI_ACCENT2_COLOR,
            TabState::Selected => colors::TRUCKUI_ACCENT_COLOR,
            TabState::Hover => colors::TRUCKUI_BGCOLOR,
            TabState::Default => colors::TRUCKUI_BGCOLOR.with_alpha(0.7),
            TabState::Disabled => colors::TRUCKUI_BGCOLOR.with_alpha(0.5),
        }
    }

    pub(crate) fn font_size(&self) -> f32 {
        match self.state {
            TabState::Selected => 35.0 * FONT_SCALE,
            _ => 24.0 * FONT_SCALE,
        }
    }
}
