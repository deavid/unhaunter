use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Orientation {
    XAxis,
    YAxis,
    Both,
    #[default]
    None,
}

impl Orientation {
    pub fn flip(&mut self) {
        match self {
            Orientation::XAxis => *self = Orientation::YAxis,
            Orientation::YAxis => *self = Orientation::XAxis,
            Orientation::Both => {}
            Orientation::None => {}
        }
    }
}
