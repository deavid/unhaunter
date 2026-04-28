use crate::identity_data::{NICKNAMES, SURNAMES};
use uuid::Uuid;

const CONSONANTS: &[char] = &[
    'B', 'C', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X',
    'Y', 'Z',
];

/// Generates a deterministic name from a UUID.
/// Format: C.C.Surname1-Surname2 (Nickname)
pub fn generate_deterministic_name(uuid: Uuid) -> String {
    let mut val = uuid.as_u128();

    let c1 = CONSONANTS[(val % CONSONANTS.len() as u128) as usize];
    val /= CONSONANTS.len() as u128;

    let c2 = CONSONANTS[(val % CONSONANTS.len() as u128) as usize];
    val /= CONSONANTS.len() as u128;

    let s1 = SURNAMES[(val % SURNAMES.len() as u128) as usize];
    val /= SURNAMES.len() as u128;

    let s2 = SURNAMES[(val % SURNAMES.len() as u128) as usize];
    val /= SURNAMES.len() as u128;

    let nickname = NICKNAMES[(val % NICKNAMES.len() as u128) as usize];

    format!("{c1}.{c2}.{s1}-{s2} ({nickname})")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_determinism() {
        let uuid = Uuid::from_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let name1 = generate_deterministic_name(uuid);
        let name2 = generate_deterministic_name(uuid);
        assert_eq!(name1, name2);
    }

    #[test]
    fn test_uniqueness() {
        let uuid1 = Uuid::from_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid2 = Uuid::from_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
        let name1 = generate_deterministic_name(uuid1);
        let name2 = generate_deterministic_name(uuid2);
        assert_ne!(name1, name2);
    }

    #[test]
    fn test_format() {
        let uuid = Uuid::new_v4();
        let name = generate_deterministic_name(uuid);
        // Format: C.C.Surname1-Surname2 (Nickname)
        // Example: B.D.Smith-Johnson (Time)
        assert!(name.contains('.'));
        assert!(name.contains('-'));
        assert!(name.contains('('));
        assert!(name.contains(')'));

        let parts: Vec<&str> = name
            .split(['.', '-', ' ', '(', ')'])
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(parts.len(), 5, "Name was: {}", name);
    }
}
