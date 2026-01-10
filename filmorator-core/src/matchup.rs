use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::hash::BuildHasher;

use crate::models::PhotoRating;

#[must_use]
pub fn generate_seed_matchups(num_photos: u32, matchup_size: usize) -> Vec<Vec<u32>> {
    let Some(size_u32) = u32::try_from(matchup_size).ok() else {
        return vec![];
    };
    if num_photos < size_u32 || num_photos == 0 {
        return vec![];
    }

    let mut rng = rand::rng();
    let mut indices: Vec<u32> = (0..num_photos).collect();
    let mut matchups = Vec::new();

    // ceil(log2(n)) + 1 rounds ensures good coverage via integer math
    let rounds = (u32::BITS - (num_photos - 1).leading_zeros()) as usize + 1;

    for _ in 0..rounds {
        indices.shuffle(&mut rng);

        for chunk in indices.chunks(matchup_size) {
            if chunk.len() == matchup_size {
                matchups.push(chunk.to_vec());
            }
        }
    }

    matchups
}

#[must_use]
pub fn select_dynamic_matchup<S: BuildHasher>(
    ratings: &[PhotoRating],
    compared_pairs: &HashSet<(u32, u32), S>,
    matchup_size: usize,
) -> Option<Vec<u32>> {
    if ratings.len() < matchup_size {
        return None;
    }

    let mut scored: Vec<(u32, f64)> = ratings
        .iter()
        .map(|r| (r.photo_idx, r.uncertainty))
        .collect();

    scored.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut selected = Vec::with_capacity(matchup_size);

    for (photo_idx, _) in &scored {
        if selected.len() >= matchup_size {
            break;
        }

        let dominated = selected
            .iter()
            .all(|&s| compared_pairs.contains(&normalize_pair(*photo_idx, s)));

        if !dominated || selected.is_empty() {
            selected.push(*photo_idx);
        }
    }

    if selected.len() < matchup_size {
        selected.clear();
        selected.extend(scored.iter().take(matchup_size).map(|(idx, _)| *idx));
    }

    (selected.len() == matchup_size).then_some(selected)
}

#[must_use]
pub const fn normalize_pair(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

#[must_use]
pub fn extract_compared_pairs(results: &[(u32, u32)]) -> HashSet<(u32, u32)> {
    results.iter().map(|&(a, b)| normalize_pair(a, b)).collect()
}

#[must_use]
pub const fn total_pairs_needed(num_photos: u32) -> u64 {
    if num_photos < 2 {
        return 0;
    }
    let n = num_photos as u64;
    n * (n - 1) / 2
}

#[must_use]
pub fn completion_percent(compared_pairs: u64, num_photos: u32) -> u8 {
    let total = total_pairs_needed(num_photos);
    if total == 0 {
        return 100;
    }
    let percent = compared_pairs.saturating_mul(100) / total;
    u8::try_from(percent.min(100)).unwrap_or(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_matchups_produces_correct_size() {
        let matchups = generate_seed_matchups(10, 3);
        assert!(!matchups.is_empty());
        for matchup in &matchups {
            assert_eq!(matchup.len(), 3);
        }
    }

    #[test]
    fn seed_matchups_cover_majority_of_pairs() {
        let matchups = generate_seed_matchups(12, 3);
        let mut seen: HashSet<(u32, u32)> = HashSet::new();

        for matchup in &matchups {
            for (i, &a) in matchup.iter().enumerate() {
                for &b in &matchup[i + 1..] {
                    seen.insert(normalize_pair(a, b));
                }
            }
        }

        let coverage_percent = completion_percent(seen.len() as u64, 12);
        assert!(
            coverage_percent >= 50,
            "Coverage should be >= 50%, got {coverage_percent}%"
        );
    }

    #[test]
    fn normalize_pair_orders_correctly() {
        assert_eq!(normalize_pair(5, 3), (3, 5));
        assert_eq!(normalize_pair(3, 5), (3, 5));
        assert_eq!(normalize_pair(2, 2), (2, 2));
    }

    #[test]
    fn total_pairs_formula_correct() {
        assert_eq!(total_pairs_needed(10), 45);
        assert_eq!(total_pairs_needed(100), 4950);
    }

    #[test]
    fn completion_percent_boundaries() {
        assert_eq!(completion_percent(0, 10), 0);
        assert_eq!(completion_percent(45, 10), 100);
        assert_eq!(completion_percent(22, 10), 48);
        assert_eq!(completion_percent(0, 0), 100);
    }
}
