//! # Random Seed and Fast RNG Module
//!
//! A lightweight random number generation system designed for performance and WASM compatibility.
//! This module provides utilities for generating random numbers using system time as a seed source
//! instead of relying on OS-level randomness, which is particularly useful in environments like WASM.

use std::{
    cell::RefCell,
    hash::{DefaultHasher, Hash, Hasher},
};

use rand::{RngCore, SeedableRng};

/// Generates a seed based on the current system time.
///
/// This function creates a seed by hashing the current system time,
/// providing a simple and fast way to get a seed that is "different enough"
/// for game use, without requiring OS-level entropy.
///
/// # Returns
///
/// A `u64` seed value derived from the current system time.
pub fn heavy_rng_seed() -> u64 {
    use bevy::utils::SystemTime;
    let now = SystemTime::now();
    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    hasher.finish()
}

/// Initializes a `SmallRng` instance with a seed based on the current system time.
///
/// This function is used when a thread needs a new random number generator
/// without requiring high-quality entropy from the OS.
///
/// # Returns
///
/// A newly seeded `SmallRng` instance.
pub fn heavy_rng() -> rand::rngs::SmallRng {
    let seed = heavy_rng_seed();
    rand::rngs::SmallRng::seed_from_u64(seed)
}

/// A struct responsible for managing the per-thread seeding process.
///
/// Each instance contains a `SmallRng` which is used to quickly seed new RNGs.
/// This provides an efficient way to generate multiple random number generators
/// without repeatedly accessing system time.
#[derive(Debug, Clone, Default)]
pub struct FastRngSeeder {
    rng: Option<rand::rngs::SmallRng>,
}

impl FastRngSeeder {
    /// Returns the thread's random number generator, initializing it if necessary.
    ///
    /// # Returns
    ///
    /// A mutable reference to the thread's `SmallRng` instance.
    pub fn rng(&mut self) -> &mut rand::rngs::SmallRng {
        if self.rng.is_none() {
            self.rng = Some(heavy_rng());
        }
        self.rng.as_mut().unwrap()
    }
}

thread_local! {
    static RNG: RefCell<FastRngSeeder> = RefCell::new(FastRngSeeder::default());
}

/// Gets a new random number generator seeded from the thread-local RNG.
///
/// This is the main function used to get a random number generator.
/// It borrows the thread's fast RNG to generate a unique seed, and uses
/// that to create a new `SmallRng`.
///
/// # Example
///
/// ```rust
/// use rand::Rng;
/// use uncore::random_seed;
///
/// let mut rng = random_seed::rng();
/// let random_number: u32 = rng.random();
/// ```
///
/// # Returns
///
/// A newly seeded `SmallRng` instance.
pub fn rng() -> rand::rngs::SmallRng {
    let seed: u64 = RNG.with(|rng| rng.borrow_mut().rng().next_u64());
    rand::rngs::SmallRng::seed_from_u64(seed)
}
