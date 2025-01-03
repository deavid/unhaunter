pub mod direction;
pub mod position;
pub mod boardposition;
pub mod mapcolor;

pub const EPSILON: f32 = 0.0001;

// old perspective (9x20cm) const SUBTL: f32 = 9.0; new perspective (3x20cm)
pub const SUBTL: f32 = 3.0;

// new perspective (3x20cm) - reduced const SUBTL: f32 = 2.5;
pub const PERSPECTIVE_X: [f32; 3] = [4.0 * SUBTL, -2.0 * SUBTL, 0.0001];
pub const PERSPECTIVE_Y: [f32; 3] = [4.0 * SUBTL, 2.0 * SUBTL, -0.0001];
pub const PERSPECTIVE_Z: [f32; 3] = [0.0, 4.0 * 11.0, 0.01];
