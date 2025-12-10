use crate::colors;
use bevy::{color::palettes::css, prelude::*};

use super::{evidence::Evidence, truck_button::TruckButtonState};

#[derive(Debug, Clone)]
pub struct EvidenceStatus {
    pub title: String,
    pub status_game: String,
    pub status_desc: String,
    pub status_color: Color,
    pub help_text: String,
}

impl EvidenceStatus {
    pub fn from_gearkind(o_evidence: Option<Evidence>, ev_state: TruckButtonState) -> Self {
        let Some(evidence) = o_evidence else {
            return Self {
                title: "".into(),
                status_game: "".into(),
                status_desc: "".into(),
                status_color: colors::INVENTORY_STATS_COLOR,
                help_text: "No evidence for selected gear.".into(),
            };
        };
        let title: String = format!("{}: ", evidence.name());
        let help_text: String = evidence.help_text().into();
        let status_game: String = match ev_state {
            TruckButtonState::Off => "[ ] Unknown\n",
            TruckButtonState::Pressed => "[+] Found\n",
            TruckButtonState::Discard => "[-] Discarded\n",
        }
        .into();
        let status_desc: String = match ev_state {
            TruckButtonState::Off => "Unknown",
            TruckButtonState::Pressed => "Found",
            TruckButtonState::Discard => "Discarded",
        }
        .into();
        let status_color: Color = match ev_state {
            TruckButtonState::Off => colors::INVENTORY_STATS_COLOR.with_alpha(1.0),
            TruckButtonState::Pressed => css::GREEN.with_alpha(1.0).into(),
            TruckButtonState::Discard => css::RED.with_alpha(0.8).into(),
        };
        Self {
            title,
            status_game,
            status_desc,
            status_color,
            help_text,
        }
    }
}
