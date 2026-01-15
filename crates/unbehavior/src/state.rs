use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TileState {
    // Switch states
    On,
    Off,
    // Door states
    Open,
    Closed,
    // Wall states
    Full,
    Partial,
    Minimum,
    // Default state
    #[default]
    None,
}
