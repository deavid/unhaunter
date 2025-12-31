use bevy_platform::collections::HashMap;

/// Mapping between floor numbers and z-coordinates
#[derive(Debug, Clone)]
pub struct FloorLevelMapping {
    /// Maps floor numbers to z-coordinates (e.g., -1 -> 0, 0 -> 1, 2 -> 2)
    pub floor_to_z: HashMap<i32, usize>,
    /// Maps z-coordinates back to floor numbers
    pub z_to_floor: HashMap<usize, i32>,
    /// Display names for each floor
    pub floor_display_names: HashMap<i32, String>,
    /// Required number of ghost attracting objects for each floor
    pub ghost_attracting_objects: HashMap<i32, i32>,
    /// Required number of ghost repelling objects for each floor
    pub ghost_repelling_objects: HashMap<i32, i32>,
}
