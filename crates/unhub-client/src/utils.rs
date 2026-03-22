use rand::prelude::*;
use sha2::{Digest, Sha256};

pub const SAFE_VOCAL_ALPHABET: &[char] = &[
    'C', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'P', 'R', 'S', 'T', 'V', 'W', 'X', '2', '4', '7',
    '9',
];

pub fn generate_room_code() -> String {
    let mut rng = rand::rng();
    (0..5)
        .map(|_| *SAFE_VOCAL_ALPHABET.choose(&mut rng).unwrap())
        .collect()
}

/// Generates a random room secret for basic access control.
/// This is NOT a cryptographic secret — it only prevents casual guessing
/// of room connections. No cryptographic security guarantees are needed.
pub fn generate_room_secret() -> String {
    let mut rng = rand::rng();
    let chars: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    (0..16)
        .map(|_| *chars.choose(&mut rng).unwrap() as char)
        .collect()
}

pub const ADJECTIVES: &[&str] = &[
    "Ancient",
    "Broken",
    "Cold",
    "Dark",
    "Eerie",
    "Forgotten",
    "Gloomy",
    "Haunted",
    "Invisible",
    "Jaded",
    "Keen",
    "Lonely",
    "Misty",
    "Nightly",
    "Old",
    "Pale",
    "Quiet",
    "Restless",
    "Silent",
    "Tragic",
    "Unknown",
    "Vague",
    "Wicked",
    "Xenon",
    "Young",
    "Zealous",
];

pub const NOUNS: &[&str] = &[
    "Apparition",
    "Banshee",
    "Cryptid",
    "Demon",
    "Entity",
    "Fragment",
    "Ghost",
    "Haint",
    "Image",
    "Jester",
    "Kestrel",
    "Lurker",
    "Manifestation",
    "Nightmare",
    "Orb",
    "Phantom",
    "Quaint",
    "Revenant",
    "Specter",
    "Trace",
    "Underworlder",
    "Vapor",
    "Wraith",
    "Xenomorph",
    "Yeti",
    "Zombie",
];

pub fn generate_codename(letter: char, attempt: u32) -> String {
    let l = letter.to_uppercase().next().unwrap_or('A');
    let adjectives_filtered: Vec<_> = ADJECTIVES
        .iter()
        .filter(|a| a.starts_with(l))
        .cloned()
        .collect();
    let nouns_filtered: Vec<_> = NOUNS.iter().filter(|n| n.starts_with(l)).cloned().collect();

    let adjectives = if adjectives_filtered.is_empty() {
        vec!["Anonymous"]
    } else {
        adjectives_filtered
    };
    let nouns = if nouns_filtered.is_empty() {
        vec!["Agent"]
    } else {
        nouns_filtered
    };

    let adj_idx = (attempt as usize) % adjectives.len();
    let noun_idx = ((attempt as usize) / adjectives.len()) % nouns.len();

    format!("{} {}", adjectives[adj_idx], nouns[noun_idx])
}

pub fn solve_pow(nonce: &str, difficulty: u32) -> String {
    let mut i = 0u64;
    loop {
        let candidate = format!("{}:{}", nonce, i);
        let hash = Sha256::digest(candidate.as_bytes());

        // Check leading zero bits
        let mut zero_bits = 0;
        for byte in hash {
            if byte == 0 {
                zero_bits += 8;
            } else {
                zero_bits += byte.leading_zeros();
                break;
            }
        }

        if zero_bits >= difficulty {
            return i.to_string();
        }
        i += 1;
    }
}
