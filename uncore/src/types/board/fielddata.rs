use super::light::LightData;

#[derive(Clone, Debug)]
pub struct LightFieldData {
    pub lux: f32,
    pub color: (f32, f32, f32),
    pub transmissivity: f32,
    pub additional: LightData,
}

impl Default for LightFieldData {
    fn default() -> Self {
        Self {
            lux: 0.0,
            color: (1.0, 1.0, 1.0),
            transmissivity: 1.0,
            additional: LightData::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Copy)]
pub struct CollisionFieldData {
    pub player_free: bool,
    pub ghost_free: bool,
    pub see_through: bool,
}
