#![cfg(test)]

use itertools::Itertools;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::ghost_definitions::Evidence;
use crate::ghost_definitions::GhostType;

const MAX_COMBO: usize = 1024 * 1024;

fn find_and_score_ghost_sets(
    wanted_evidence: &HashSet<Evidence>,
    num_ghosts: usize,
) -> Vec<(HashSet<GhostType>, i32)> {
    let all_evidence: HashSet<Evidence> = Evidence::all().collect();
    let unwanted_evidence: HashSet<Evidence> =
        all_evidence.difference(wanted_evidence).cloned().collect();
    let unwanted_evidence_count = unwanted_evidence.len() as u32;
    let all_ghosts = GhostType::all().collect::<Vec<GhostType>>();
    let min_presence = num_ghosts / 2;
    let min_absence = num_ghosts / 4;

    let mut results: Vec<(HashSet<GhostType>, i32)> = Vec::new();

    for unwanted_evidence_bitset in 0..2usize.pow(unwanted_evidence_count) {
        let filtered_ghosts: Vec<GhostType> = all_ghosts
            .iter()
            .cloned()
            .filter(|ghost| {
                for (i, evidence) in unwanted_evidence.iter().enumerate() {
                    let bit_is_set = (unwanted_evidence_bitset >> i) & 1 == 1;
                    let has_evidence = ghost.evidences().contains(evidence);
                    if bit_is_set && !has_evidence {
                        return false;
                    }
                    if !bit_is_set && has_evidence {
                        return false;
                    }
                }
                true
            })
            .collect();

        if filtered_ghosts.len() < num_ghosts {
            continue;
        }

        let mut evidence_present_tally: HashMap<Evidence, usize> = HashMap::new();
        let mut evidence_absent_tally: HashMap<Evidence, usize> = HashMap::new();
        for ghost in &filtered_ghosts {
            for evidence in wanted_evidence.iter() {
                if ghost.evidences().contains(evidence) {
                    *evidence_present_tally.entry(*evidence).or_insert(0) += 1;
                } else {
                    *evidence_absent_tally.entry(*evidence).or_insert(0) += 1;
                }
            }
        }

        if wanted_evidence
            .iter()
            .any(|e| evidence_present_tally.get(e).unwrap_or(&0) < &min_presence)
        {
            continue;
        }
        if wanted_evidence
            .iter()
            .any(|e| evidence_absent_tally.get(e).unwrap_or(&0) < &min_absence)
        {
            continue;
        }

        let mut good_ghosts: HashSet<GhostType> = HashSet::new();
        let mut choice_ghosts: HashSet<GhostType> = HashSet::new();
        for &ghost in &filtered_ghosts {
            let mut too_frequent = false;
            let mut too_infrequent = false;
            for &evidence in wanted_evidence {
                if ghost.evidences().contains(&evidence)
                    && *evidence_present_tally.get(&evidence).unwrap_or(&0) > min_presence
                {
                    too_frequent = true;
                }
                if !ghost.evidences().contains(&evidence)
                    && *evidence_absent_tally.get(&evidence).unwrap_or(&0) > min_absence
                {
                    too_infrequent = true;
                }
            }

            if too_frequent || too_infrequent {
                choice_ghosts.insert(ghost);
            } else {
                good_ghosts.insert(ghost);
            }
        }

        if good_ghosts.len() >= num_ghosts {
            for combination in good_ghosts.iter().combinations(num_ghosts).take(MAX_COMBO) {
                let ghost_set: HashSet<GhostType> = combination.into_iter().cloned().collect();
                if is_uniquely_identifiable(&ghost_set, wanted_evidence) {
                    let score = score_ghost_set(&ghost_set, wanted_evidence, num_ghosts);
                    results.push((ghost_set, score));
                }
            }
            continue;
        } else {
            let remaining_ghosts = num_ghosts - good_ghosts.len();
            for choice_combination in choice_ghosts
                .iter()
                .combinations(remaining_ghosts)
                .take(MAX_COMBO)
            {
                let mut combined_set = good_ghosts.clone();
                combined_set.extend(choice_combination.into_iter().cloned());

                let mut combined_present_tally: HashMap<Evidence, usize> = HashMap::new();
                let mut combined_absent_tally: HashMap<Evidence, usize> = HashMap::new();
                for ghost in &combined_set {
                    for evidence in wanted_evidence.iter() {
                        if ghost.evidences().contains(evidence) {
                            *combined_present_tally.entry(*evidence).or_insert(0) += 1;
                        } else {
                            *combined_absent_tally.entry(*evidence).or_insert(0) += 1;
                        }
                    }
                }

                if wanted_evidence
                    .iter()
                    .any(|e| combined_present_tally.get(e).unwrap_or(&0) < &min_presence)
                {
                    continue;
                }
                if wanted_evidence
                    .iter()
                    .any(|e| combined_absent_tally.get(e).unwrap_or(&0) < &min_absence)
                {
                    continue;
                }

                if is_uniquely_identifiable(&combined_set, wanted_evidence) {
                    let score = score_ghost_set(&combined_set, wanted_evidence, num_ghosts);
                    results.push((combined_set, score));
                }
            }
        }
    }

    results.sort_by_key(|&(_, score)| -score);
    results.truncate(10);
    results
}

fn is_uniquely_identifiable(
    ghost_set: &HashSet<GhostType>,
    wanted_evidence: &HashSet<Evidence>,
) -> bool {
    let unique_evidence_sets: HashSet<u8> = ghost_set
        .iter()
        .map(|ghost| {
            let mut evidence_bitset = 0;
            for (i, evidence) in Evidence::all().enumerate() {
                if wanted_evidence.contains(&evidence) && ghost.evidences().contains(&evidence) {
                    evidence_bitset |= 1 << i;
                }
            }
            evidence_bitset
        })
        .collect();

    unique_evidence_sets.len() == ghost_set.len()
}

fn score_ghost_set(
    ghost_set: &HashSet<GhostType>,
    wanted_evidence: &HashSet<Evidence>,
    num_ghosts: usize,
) -> i32 {
    let mut score = 20;

    let mut evidence_counts: HashMap<Evidence, usize> = HashMap::new();
    for &evidence in wanted_evidence {
        evidence_counts.insert(evidence, 0);
    }
    for ghost in ghost_set {
        for &evidence in wanted_evidence {
            if ghost.evidences().contains(&evidence) {
                *evidence_counts.get_mut(&evidence).unwrap() += 1;
            }
        }
    }

    let mean_count = evidence_counts.values().sum::<usize>() as f64 / evidence_counts.len() as f64;
    let max_deviation = evidence_counts
        .values()
        .map(|&count| (count as f64 - mean_count).abs())
        .fold(0.0, f64::max);

    score -= max_deviation.ceil() as i32;

    let ideal_total_count = num_ghosts as f64 * wanted_evidence.len() as f64 * (2.0 / 3.0);
    let actual_total_count = evidence_counts.values().sum::<usize>() as f64;
    score -= (ideal_total_count - actual_total_count).abs() as i32;

    score
}

#[test]
fn test_find_and_score_ghost_sets() {
    let wanted_evidence = vec![
        Evidence::FreezingTemp,
        Evidence::EMFLevel5,
        Evidence::UVEctoplasm,
        Evidence::FloatingOrbs,
        Evidence::EVPRecording,
        Evidence::CPM500,
        Evidence::SpiritBox,
        Evidence::RLPresence,
    ]
    .into_iter()
    .collect();

    let result = find_and_score_ghost_sets(&wanted_evidence, 20);
    println!("{result:?}");
    assert!(!result.is_empty());
}
