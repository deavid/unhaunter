use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProgressionData {
    pub bank: i64,
    #[serde(default)]
    pub insurance_deposit: i64,
    #[serde(default)]
    pub player_xp: i64,
    #[serde(default)]
    pub player_level: i32,
}

impl Default for ProgressionData {
    fn default() -> Self {
        Self {
            bank: 0,
            insurance_deposit: 0,
            player_xp: 0,
            player_level: 1,
        }
    }
}

impl ProgressionData {
    /// Calculates the player level based on player_xp.
    pub fn calculate_player_level(player_xp: i64) -> f64 {
        let n: f64 = 250.0;
        let k: f64 = 1.5;
        (player_xp as f64 / n + 1.0).powf(k.recip())
    }

    /// Updates the player_level field based on the current player_xp.
    pub fn update_level(&mut self) {
        self.player_level = Self::calculate_player_level(self.player_xp).floor() as i32;
    }

    /// Gets the current progress towards the next level as a float between 0.0 and 1.0.
    pub fn get_level_progress(&self) -> f32 {
        (Self::calculate_player_level(self.player_xp).fract()) as f32
    }

    /// Updates the player insurance deposit and bank balance.
    /// Returns an error message if the bank balance is insufficient.
    pub fn update_deposit(&mut self, required_deposit: i64) -> Result<(), String> {
        let additional_needed = required_deposit - self.insurance_deposit;

        match additional_needed.cmp(&0) {
            std::cmp::Ordering::Greater => {
                if self.bank >= additional_needed {
                    self.bank -= additional_needed;
                    self.insurance_deposit += additional_needed;
                } else {
                    return Err(format!(
                        "Insufficient Money in Bank for deposit. Required: ${}, Available: ${}",
                        required_deposit, self.bank
                    ));
                }
            }
            std::cmp::Ordering::Less => {
                let refund = -additional_needed;
                self.bank += refund;
                self.insurance_deposit -= refund;
            }
            std::cmp::Ordering::Equal => {}
        }

        Ok(())
    }
}
