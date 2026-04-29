//! Deterministic name pool data.
use once_cell::sync::Lazy;

const SURNAMES_RAW: &str = include_str!("../../../bundled/identity/surnames.txt");
const NICKNAMES_RAW: &str = include_str!("../../../bundled/identity/nicknames.txt");

pub static SURNAMES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    SURNAMES_RAW
        .lines()
        .filter(|line| !line.is_empty())
        .collect()
});

pub static NICKNAMES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    NICKNAMES_RAW
        .lines()
        .filter(|line| !line.is_empty())
        .collect()
});
