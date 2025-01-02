pub use uncore::types::ghost_type::GhostType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GhostSet {
    TmpEMF,
    TmpEMFUVOrbs,
    TmpEMFUVOrbsEVPCPM,
    Twenty,
    #[default]
    All,
}

impl GhostSet {
    pub fn as_vec(&self) -> Vec<GhostType> {
        use GhostType::*;

        match self {
            Self::TmpEMF => vec![LadyInWhite, BrownLady],
            Self::TmpEMFUVOrbs => vec![Caoilte, Ceara, Orla, Finvarra, Kappa],
            Self::TmpEMFUVOrbsEVPCPM => vec![
                Bugbear, Morag, Barghest, Boggart, Obayifo, WillOWisp, LaLlorona, Widow,
                Leprechaun, Brume,
            ],
            Self::Twenty => vec![
                Curupira, LaLlorona, Phooka, Obayifo, Maresca, Dybbuk, Caoilte, Orla, Jorogumo,
                Mider, Wisp, Cairbre, Ceara, Widow, BeanSidhe, Bugbear, Dullahan, Domovoi,
                Muirgheas, Namahage,
            ],
            Self::All => GhostType::all().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::utils::{HashMap, HashSet};
    use enum_iterator::all;
    use uncore::types::evidence::Evidence;

    #[test]
    fn test_generate_evidence_combinations() {
        for i in 0..256 {
            let mut nbits = 0;
            for n in 0..8 {
                if (i >> n) & 0x1 > 0 {
                    nbits += 1;
                }
            }
            if nbits == 5 {
                println!("0b{:08b}", i);
            }
        }
    }

    #[test]
    fn test_unique_evidence_combinations() {
        let mut all_combinations: HashSet<String> = HashSet::new();
        for ghost in all::<GhostType>() {
            let mut evidences = ghost
                .evidences()
                .into_iter()
                .map(|x| x.name())
                .collect::<Vec<_>>();
            evidences.sort();
            let evidences = evidences.join("|");
            assert!(
                all_combinations.insert(evidences),
                "Found duplicate evidence set for {:?}",
                ghost
            );
        }
    }

    #[test]
    fn test_evidence_per_ghost() {
        for ghost in all::<GhostType>() {
            let evidences = ghost
                .evidences()
                .into_iter()
                .map(|x| x.name())
                .collect::<Vec<_>>();
            assert!(
                evidences.len() == 5,
                "The ghost {:?} does not have 5 evidences, instead it has: {:?}",
                ghost,
                evidences
            );
        }
    }

    #[test]
    fn test_balanced_evidence_usage() {
        let mut evidence_count: HashMap<Evidence, usize> = HashMap::new();
        for ghost in all::<GhostType>() {
            for &evidence in &ghost.evidences() {
                *evidence_count.entry(evidence).or_insert(0) += 1;
            }
        }

        // Assuming a balanced distribution, each evidence should be used roughly the same
        // number of times.
        let avg_use = evidence_count.values().sum::<usize>() / evidence_count.len();
        for (&evidence, &count) in &evidence_count {
            println!("Evidence {:?} used {} times", evidence, count);
        }
        for (&evidence, &count) in &evidence_count {
            assert!(
                (count as i32 - avg_use as i32).abs() <= 3,
                "Evidence {:?} is used an unbalanced number of times: {} (avg: {})",
                evidence,
                count,
                avg_use
            );
        }
    }
}
