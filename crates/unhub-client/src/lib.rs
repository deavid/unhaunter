pub mod protocol;
pub mod tickets;
pub mod utils;

pub const GAME_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use utils::{
    ADJECTIVES, NOUNS, SAFE_VOCAL_ALPHABET, generate_codename, generate_room_code,
    generate_room_secret, solve_pow, solve_pow_async,
};
