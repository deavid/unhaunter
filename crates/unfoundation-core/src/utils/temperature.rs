// Temperature conversion utilities
pub const KELVIN_OFFSET: f32 = 273.15;

#[inline]
pub fn celsius_to_kelvin(celsius: f32) -> f32 {
    celsius + KELVIN_OFFSET
}

#[inline]
pub fn kelvin_to_celsius(kelvin: f32) -> f32 {
    kelvin - KELVIN_OFFSET
}
