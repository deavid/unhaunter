use bevy::prelude::*;

use crate::types::truck_button::{TruckButtonState, TruckButtonType};
use uninvestigation_core::evidence::Evidence;

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
    /// Whether the button is locked computer side, meaning it has a value already that cannot be changed anymore.
    pub computer_locked: bool,
}

impl TruckUIButton {
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
            frame_counter: 0,
            computer_locked: false,
        }
    }
}
