use bevy::prelude::*;

use crate::colors;
use crate::events::truck::TruckUIEvent;
use crate::types::truck_button::{TruckButtonState, TruckButtonType};

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
}

impl TruckUIButton {
    pub fn pressed(&mut self) -> Option<TruckUIEvent> {
        match self.class {
            TruckButtonType::Evidence(_) | TruckButtonType::Ghost(_) => {
                self.status = match self.status {
                    TruckButtonState::Off => TruckButtonState::Pressed,
                    TruckButtonState::Pressed => TruckButtonState::Discard,
                    TruckButtonState::Discard => TruckButtonState::Off,
                };
                None
            }
            TruckButtonType::CraftRepellent => Some(TruckUIEvent::CraftRepellent),
            TruckButtonType::ExitTruck => Some(TruckUIEvent::ExitTruck),
            TruckButtonType::EndMission => Some(TruckUIEvent::EndMission),
        }
    }

    pub fn border_color(&self, interaction: Interaction) -> Color {
        let color = match self.class {
            TruckButtonType::Evidence(_) => match interaction {
                Interaction::Pressed => colors::TRUCKUI_ACCENT3_COLOR,
                Interaction::Hovered => colors::TRUCKUI_TEXT_COLOR,
                Interaction::None => colors::TRUCKUI_ACCENT2_COLOR,
            },
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

    pub fn background_color(&self, interaction: Interaction) -> Color {
        let color = match self.class {
            TruckButtonType::Evidence(_) => match self.status {
                TruckButtonState::Off => colors::TRUCKUI_BGCOLOR,
                TruckButtonState::Pressed => colors::TRUCKUI_ACCENT2_COLOR,
                TruckButtonState::Discard => colors::BUTTON_END_MISSION_FGCOLOR,
            },
            TruckButtonType::Ghost(_) => match self.status {
                TruckButtonState::Off => colors::TRUCKUI_BGCOLOR.with_alpha(0.0),
                TruckButtonState::Pressed => colors::TRUCKUI_ACCENT2_COLOR,
                TruckButtonState::Discard => colors::BUTTON_END_MISSION_FGCOLOR,
            },
            TruckButtonType::ExitTruck | TruckButtonType::CraftRepellent => match interaction {
                Interaction::Pressed => colors::BUTTON_EXIT_TRUCK_FGCOLOR,
                Interaction::Hovered => colors::BUTTON_EXIT_TRUCK_BGCOLOR,
                Interaction::None => colors::BUTTON_EXIT_TRUCK_BGCOLOR,
            },
            TruckButtonType::EndMission => match interaction {
                Interaction::Pressed => colors::BUTTON_END_MISSION_FGCOLOR,
                Interaction::Hovered => colors::BUTTON_END_MISSION_BGCOLOR,
                Interaction::None => colors::BUTTON_END_MISSION_BGCOLOR,
            },
        };
        let alpha_disabled = if self.disabled { 0.05 } else { 1.0 };
        color.with_alpha(color.alpha() * alpha_disabled)
    }

    pub fn text_color(&self, _interaction: Interaction) -> Color {
        let color = match self.class {
            TruckButtonType::Evidence(_) => match self.status {
                TruckButtonState::Pressed => Color::BLACK,
                _ => colors::TRUCKUI_TEXT_COLOR,
            },
            TruckButtonType::Ghost(_) => match self.status {
                TruckButtonState::Pressed => Color::BLACK,
                _ => colors::TRUCKUI_TEXT_COLOR.with_alpha(0.5),
            },
            TruckButtonType::ExitTruck | TruckButtonType::CraftRepellent => {
                colors::BUTTON_EXIT_TRUCK_TXTCOLOR
            }
            TruckButtonType::EndMission => colors::BUTTON_END_MISSION_TXTCOLOR,
        };
        let alpha_disabled = if self.disabled { 0.1 } else { 1.0 };
        color.with_alpha(color.alpha() * alpha_disabled)
    }
}

impl From<TruckButtonType> for TruckUIButton {
    fn from(value: TruckButtonType) -> Self {
        TruckUIButton {
            status: TruckButtonState::Off,
            class: value,
            disabled: false,
        }
    }
}
