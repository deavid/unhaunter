use anyhow::Context;
use serde::{Deserialize, Serialize};

pub trait AutoSerialize: Serialize + for<'a> Deserialize<'a> + Default {
    fn from_text(text: Option<&str>) -> anyhow::Result<Self> {
        let Some(text) = text else {
            return Ok(Self::default());
        };
        let t = format!("\"{text}\"");
        serde_json::from_str(&t).context("Auto deserialize error")
    }

    #[allow(dead_code)]
    fn to_text(&self) -> anyhow::Result<String> {
        // FIXME: This is not used at all.
        serde_json::to_string(self)
            .map(|x| x.replace('"', ""))
            .context("Auto serialize error")
    }
}

use unspatial_core::orientation::Orientation;
impl AutoSerialize for Orientation {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::class::Class;
    use crate::state::TileState;

    #[test]
    fn class_serialization() {
        let c = Class::Breaker;
        let ct = c.to_text().unwrap();
        assert_eq!(ct, "Breaker");
        let d = Class::from_text(Some(&ct)).unwrap();
        assert_eq!(d, c);
    }

    #[test]
    fn state_serialization() {
        let c = TileState::On;
        let ct = c.to_text().unwrap();
        assert_eq!(ct, "On");
        let d = TileState::from_text(Some(&ct)).unwrap();
        assert_eq!(d, c);
    }
}
