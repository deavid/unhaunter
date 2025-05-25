use bevy::prelude::*;

use crate::colors;
use crate::events::truck::TruckUIEvent;
use crate::types::evidence::Evidence;
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
    /// Duration in seconds the button must be held to activate (None = instant)
    pub hold_duration: Option<f32>,
    /// Current time the button has been held
    pub hold_timer: Option<f32>,
    /// Whether the button is currently being held
    pub holding: bool,
    /// Whether this button should blink to indicate a hint (for evidence buttons)
    pub blinking_hint_active: bool,
    /// Frame counter for blinking animation
    pub frame_counter: u32,
}

impl TruckUIButton {
    pub fn pressed(&mut self) -> Option<TruckUIEvent> {
        // If this button requires holding, don't trigger immediately
        if self.hold_duration.is_some() {
            return None;
        }

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

    pub fn background_color(&self, interaction: Interaction) -> Color {
        let color = match self.class {
            TruckButtonType::Evidence(_) => match self.status {
                TruckButtonState::Off => colors::TRUCKUI_BGCOLOR,
                TruckButtonState::Pressed => Color::srgb(0.2, 0.8, 0.3), // Green color for confirmed evidence
                TruckButtonState::Discard => colors::BUTTON_END_MISSION_FGCOLOR,
            },
            TruckButtonType::Ghost(_) => match self.status {
                TruckButtonState::Off => colors::TRUCKUI_BGCOLOR.with_alpha(0.0),
                TruckButtonState::Pressed => Color::srgb(0.2, 0.8, 0.3), // Green color for confirmed ghost, same as evidence
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

    /// Update the frame counter for blinking animation
    pub fn update_frame_counter(&mut self) {
        self.frame_counter = self.frame_counter.wrapping_add(1);
    }

    /// Set blinking hint state for evidence buttons
    pub fn set_blinking_hint(&mut self, active: bool) {
        self.blinking_hint_active = active;
    }

    /// Get the evidence type if this is an evidence button
    pub fn get_evidence(&self) -> Option<Evidence> {
        if let TruckButtonType::Evidence(evidence) = &self.class {
            Some(*evidence)
        } else {
            None
        }
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
            frame_counter: 0,
        }
    }
}
