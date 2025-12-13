pub mod chunk;
pub mod mapcolor;

// Re-export spatial types from uncore-board (single source of truth)
pub use uncore_board::components::boardposition::BoardPosition;
pub use uncore_board::components::direction::Direction;
pub use uncore_board::components::position::Position;
pub use uncore_board::components::{EPSILON, PERSPECTIVE_X, PERSPECTIVE_Y, PERSPECTIVE_Z, SUBTL};
