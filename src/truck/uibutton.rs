use bevy::prelude::*;
use std::ops::Mul;

use crate::{
    colors,
    ghost_definitions::{self},
};

use super::TruckUIEvent;

#[derive(Debug, PartialEq, Eq)]
pub enum TruckButtonType {
    Evidence(ghost_definitions::Evidence),
    Ghost(ghost_definitions::GhostType),
    CraftRepellent,
    ExitTruck,
    EndMission,
}

impl TruckButtonType {
    pub fn into_component(self) -> TruckUIButton {
        TruckUIButton::from(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruckButtonState {
    Off,
    Pressed,
    Discard,
}
#[derive(Component, Debug)]
pub struct TruckUIButton {
    pub status: TruckButtonState,
    pub class: TruckButtonType,
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
        match self.class {
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
        }
        .mul(
            Color::WHITE
                .with_a(if self.disabled { 0.05 } else { 1.0 })
                .as_rgba_f32(),
        )
    }
    pub fn background_color(&self, interaction: Interaction) -> Color {
        match self.class {
            TruckButtonType::Evidence(_) => match self.status {
                TruckButtonState::Off => colors::TRUCKUI_BGCOLOR,
                TruckButtonState::Pressed => colors::TRUCKUI_ACCENT2_COLOR,
                TruckButtonState::Discard => colors::BUTTON_END_MISSION_FGCOLOR,
            },
            TruckButtonType::Ghost(_) => match self.status {
                TruckButtonState::Off => colors::TRUCKUI_BGCOLOR.with_a(0.0),
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
        }
        .mul(
            Color::WHITE
                .with_a(if self.disabled { 0.05 } else { 1.0 })
                .as_rgba_f32(),
        )
    }

    pub fn text_color(&self, _interaction: Interaction) -> Color {
        match self.class {
            TruckButtonType::Evidence(_) => match self.status {
                TruckButtonState::Pressed => Color::BLACK,
                _ => colors::TRUCKUI_TEXT_COLOR,
            },
            TruckButtonType::Ghost(_) => match self.status {
                TruckButtonState::Pressed => Color::BLACK,
                _ => colors::TRUCKUI_TEXT_COLOR.with_a(0.5),
            },
            TruckButtonType::ExitTruck | TruckButtonType::CraftRepellent => {
                colors::BUTTON_EXIT_TRUCK_TXTCOLOR
            }
            TruckButtonType::EndMission => colors::BUTTON_END_MISSION_TXTCOLOR,
        }
        .mul(
            Color::WHITE
                .with_a(if self.disabled { 0.1 } else { 1.0 })
                .as_rgba_f32(),
        )
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
