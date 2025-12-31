use bevy::prelude::*;

use unfoundation_core::colors;
use untruck_core::types::truck_button::{TruckButtonState, TruckButtonType};

/// Represents a button in the truck UI, handling its state, type, and visual
/// appearance.
#[derive(Component, Debug)]
pub struct TruckUIButton {
    /// The current state of the button.
    pub status: TruckButtonState,
    /// The type of button, determining its functionality and visual style.
    pub class: TruckButtonType,
    /// Indicates whether the button is disabled and cannot be interacted with.
    pub disabled: bool,
    /// Duration in seconds the button must be held to activate (None = instant)
    pub hold_duration: Option<f32>,
    /// Current time the button has been held
    pub hold_timer: Option<f32>,
    /// Whether the button is currently being held
    pub holding: bool,
    /// Whether this button should blink to indicate a hint (for evidence buttons)
    pub blinking_hint_active: bool,
}

impl TruckUIButton {
    pub fn border_color(&self, interaction: Interaction) -> Color {
        let color = match self.class {
            TruckButtonType::Evidence(_) => {
                // Border color for evidence buttons (blinking handled by journal system)
                match interaction {
                    Interaction::Pressed => colors::TRUCKUI_ACCENT3_COLOR,
                    Interaction::Hovered => colors::TRUCKUI_TEXT_COLOR,
                    Interaction::None => colors::TRUCKUI_ACCENT2_COLOR,
                }
            }
            TruckButtonType::Ghost(_) => match interaction {
                Interaction::Pressed => colors::TRUCKUI_ACCENT3_COLOR,
                Interaction::Hovered => colors::TRUCKUI_ACCENT_COLOR,
                Interaction::None => Color::NONE,
            },
            TruckButtonType::ExitTruck | TruckButtonType::CraftRepellent => match interaction {
                Interaction::Pressed => colors::BUTTON_EXIT_TRUCK_TXTCOLOR,
                Interaction::Hovered => colors::BUTTON_EXIT_TRUCK_TXTCOLOR,
                Interaction::None => colors::BUTTON_EXIT_TRUCK_FGCOLOR,
            },
            TruckButtonType::EndMission => match interaction {
                Interaction::Pressed => colors::BUTTON_END_MISSION_TXTCOLOR,
                Interaction::Hovered => colors::BUTTON_END_MISSION_TXTCOLOR,
                Interaction::None => colors::BUTTON_END_MISSION_FGCOLOR,
            },
        };
        let alpha_disabled = if self.disabled { 0.05 } else { 1.0 };
        color.with_alpha(color.alpha() * alpha_disabled)
    }

    /// Set blinking hint state for evidence buttons
    pub fn set_blinking_hint(&mut self, active: bool) {
        self.blinking_hint_active = active;
    }
}

/// Extension trait for TruckButtonType to convert it into a TruckUIButton component.
pub trait TruckButtonTypeExt {
    fn into_component(self) -> TruckUIButton;
}

impl TruckButtonTypeExt for TruckButtonType {
    /// Creates a `TruckUIButton` component from a `TruckButtonType`.
    fn into_component(self) -> TruckUIButton {
        TruckUIButton::from(self)
    }
}

impl From<TruckButtonType> for TruckUIButton {
    fn from(value: TruckButtonType) -> Self {
        let hold_duration = match value {
            TruckButtonType::CraftRepellent | TruckButtonType::EndMission => Some(1.0),
            _ => None,
        };

        TruckUIButton {
            status: TruckButtonState::Off,
            class: value,
            disabled: false,
            hold_duration,
            hold_timer: None,
            holding: false,
            blinking_hint_active: false,
        }
    }
}
