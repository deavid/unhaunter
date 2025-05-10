use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// Represents mission performance grades, ordered from highest (A) to lowest (N/A)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Grade {
    A,
    B,
    C,
    D,
    F,
    #[default]
    NA, // N/A (Not Applicable)
}

impl Grade {
    /// Returns the multiplier associated with each grade
    pub fn multiplier(&self) -> f64 {
        match self {
            Grade::A => 5.0,
            Grade::B => 3.0,
            Grade::C => 2.0,
            Grade::D => 1.0,
            Grade::F => 0.5,
            Grade::NA => 0.0,
        }
    }

    /// Converts a score to a Grade based on thresholds
    pub fn from_score(
        score: i64,
        a_threshold: i64,
        b_threshold: i64,
        c_threshold: i64,
        d_threshold: i64,
    ) -> Self {
        if score >= a_threshold {
            Grade::A
        } else if score >= b_threshold {
            Grade::B
        } else if score >= c_threshold {
            Grade::C
        } else if score >= d_threshold {
            Grade::D
        } else {
            Grade::F
        }
    }
}

impl PartialOrd for Grade {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Grade {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare directly based on multiplier values (higher multiplier = higher grade)
        // Multiplied by 100 to convert to integers for consistent comparison
        let self_val = (self.multiplier() * 100.0) as i32;
        let other_val = (other.multiplier() * 100.0) as i32;

        // Compare (higher value means higher grade)
        self_val.cmp(&other_val)
    }
}

impl fmt::Display for Grade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Grade::A => write!(f, "A"),
            Grade::B => write!(f, "B"),
            Grade::C => write!(f, "C"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
            Grade::NA => write!(f, "N/A"),
        }
    }
}

impl From<&str> for Grade {
    fn from(s: &str) -> Self {
        match s {
            "A" => Grade::A,
            "B" => Grade::B,
            "C" => Grade::C,
            "D" => Grade::D,
            "F" => Grade::F,
            _ => Grade::NA,
        }
    }
}

impl From<String> for Grade {
    fn from(s: String) -> Self {
        Grade::from(s.as_str())
    }
}
