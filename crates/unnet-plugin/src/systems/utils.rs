use bevy::prelude::*;
use ungearitems_core::components::flashlight::Flashlight;
use unnet_core::messages::GearDetails;

#[derive(Component, Debug)]
pub(crate) struct LastSyncedGearState {
    pub is_on: bool,
    pub details: GearDetails,
}

pub(crate) fn extract_gear_details(
    flashlight: Option<&Flashlight>,
    sage: Option<&ungearitems_core::components::sage::SageBundleData>,
    repellent: Option<&ungearitems_core::components::repellentflask::RepellentFlask>,
    thermometer: Option<&ungearitems_core::components::thermometer::Thermometer>,
    emfm: Option<&ungearitems_core::components::emfmeter::EMFMeter>,
    spiritbox: Option<&ungearitems_core::components::spiritbox::SpiritBox>,
) -> GearDetails {
    if let Some(f) = flashlight {
        GearDetails::Flashlight(f.status.clone())
    } else if let Some(s) = sage {
        GearDetails::Sage {
            consumed: s.consumed,
            is_active: s.is_active,
            remaining_secs: s.burn_timer.remaining_secs(),
        }
    } else if let Some(r) = repellent {
        GearDetails::RepellentFlask {
            qty: r.qty,
            active: r.active,
            liquid_content: r.liquid_content,
        }
    } else if let Some(t) = thermometer {
        GearDetails::Thermometer { temp: t.temp }
    } else if let Some(e) = emfm {
        GearDetails::EMF { level: e.emf }
    } else if let Some(s) = spiritbox {
        GearDetails::SpiritBox {
            charge: s.charge,
            ghost_answer: s.ghost_answer,
        }
    } else {
        GearDetails::None
    }
}
