use bevy::prelude::*;

use crate::ghosts::GhostType;

use super::{Gear, GearKind, GearSpriteID, GearUsable};

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct RepellentFlask {
    pub liquid_content: Option<GhostType>,
}

impl GearUsable for RepellentFlask {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.liquid_content.is_some() {
            true => GearSpriteID::RepelentFlaskFull,
            false => GearSpriteID::RepelentFlaskEmpty,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Unhaunter Repellent"
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = match self.liquid_content {
            Some(x) => format!("\nAnti-{} essence", x.name()),
            None => "Empty".to_string(),
        };
        let msg = if self.liquid_content.is_some() {
            "Activate in ghost room to expel it".to_string()
        } else {
            "Flask must be filled on the van".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        self.liquid_content = None;
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<RepellentFlask> for Gear {
    fn from(value: RepellentFlask) -> Self {
        Gear::new_from_kind(GearKind::RepellentFlask(value))
    }
}
