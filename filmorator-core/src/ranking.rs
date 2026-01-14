use crate::models::PhotoRating;

/// Bradley-Terry model for pairwise comparison ranking.
///
/// # Mathematical Background
///
/// Each item i has a latent strength parameter θᵢ > 0. The probability that
/// item i beats item j in a pairwise comparison is:
///
/// ```text
/// P(i beats j) = θᵢ / (θᵢ + θⱼ)
/// ```
///
/// Equivalently, using log-strengths (what we store):
///
/// ```text
/// P(i beats j) = 1 / (1 + exp(-(log θᵢ - log θⱼ)))
/// ```
///
/// This is equivalent to logistic regression on pairwise outcomes.
///
/// # MM Algorithm
///
/// We use the Minorization-Maximization algorithm for maximum likelihood
/// estimation. The update rule is:
///
/// ```text
/// θᵢ_new = wins_i / Σⱼ (n_ij / (θᵢ + θⱼ))
/// ```
///
/// where:
/// - `wins_i` = total wins for item i across all comparisons
/// - `n_ij` = number of comparisons between items i and j
///
/// After each iteration, strengths are normalized to sum to N (number of items).
/// Convergence is typically fast (50 iterations sufficient for most cases).
///
/// # Multi-way Comparisons
///
/// A 3-way ranking (A > B > C) expands to 3 pairwise outcomes:
/// - A beats B
/// - A beats C
/// - B beats C
///
/// See [`crate::models::ComparisonResult::to_pairwise`] for the expansion.
///
/// # References
///
/// - Bradley, R. A., & Terry, M. E. (1952). "Rank Analysis of Incomplete Block Designs"
/// - Hunter, D. R. (2004). "MM algorithms for generalized Bradley-Terry models"
pub struct BradleyTerry {
    num_items: u32,
    wins: Vec<Vec<u32>>,
    comparisons: Vec<Vec<u32>>,
}

impl BradleyTerry {
    /// Creates a new model. Returns `None` if `num_items` exceeds `u32::MAX`.
    #[must_use]
    pub fn new(num_items: usize) -> Option<Self> {
        let num_items_u32 = u32::try_from(num_items).ok()?;
        Some(Self {
            num_items: num_items_u32,
            wins: vec![vec![0; num_items]; num_items],
            comparisons: vec![vec![0; num_items]; num_items],
        })
    }

    pub fn record_comparison(&mut self, winner: u32, loser: u32) {
        let w = winner as usize;
        let l = loser as usize;
        let n = self.num_items as usize;
        if w < n && l < n {
            self.wins[w][l] += 1;
            self.comparisons[w][l] += 1;
            self.comparisons[l][w] += 1;
        }
    }

    pub fn record_comparisons(&mut self, results: &[(u32, u32)]) {
        for &(winner, loser) in results {
            self.record_comparison(winner, loser);
        }
    }

    #[must_use]
    pub fn compute_ratings(&self, iterations: u32) -> Vec<PhotoRating> {
        let n = self.num_items as usize;
        if n == 0 {
            return Vec::new();
        }

        let mut strengths: Vec<f64> = vec![1.0; n];

        for _ in 0..iterations {
            let mut new_strengths = vec![0.0; n];

            for i in 0..n {
                let total_wins: u32 = self.wins[i].iter().sum();
                if total_wins == 0 {
                    new_strengths[i] = strengths[i];
                    continue;
                }

                let mut denominator = 0.0;
                for j in 0..n {
                    if i != j {
                        let n_ij = f64::from(self.comparisons[i][j]);
                        if n_ij > 0.0 {
                            denominator += n_ij / (strengths[i] + strengths[j]);
                        }
                    }
                }

                new_strengths[i] = if denominator > 0.0 {
                    f64::from(total_wins) / denominator
                } else {
                    strengths[i]
                };
            }

            let sum: f64 = new_strengths.iter().sum();
            if sum > 0.0 {
                let scale = f64::from(self.num_items) / sum;
                for s in &mut new_strengths {
                    *s *= scale;
                }
            }

            strengths = new_strengths;
        }

        let mut ratings: Vec<PhotoRating> = strengths
            .into_iter()
            .enumerate()
            .filter_map(|(idx, strength)| {
                Some(PhotoRating {
                    photo_idx: u32::try_from(idx).ok()?,
                    strength: strength.ln(),
                    uncertainty: self.compute_uncertainty(idx),
                })
            })
            .collect();

        ratings.sort_by(|a, b| b.strength.total_cmp(&a.strength));
        ratings
    }

    fn compute_uncertainty(&self, item_idx: usize) -> f64 {
        let total: u32 = self.comparisons[item_idx].iter().sum();
        if total == 0 {
            return 1.0;
        }
        1.0 / (1.0 + f64::from(total).sqrt())
    }

    #[must_use]
    pub fn total_comparisons(&self) -> u64 {
        let sum: u64 = self
            .comparisons
            .iter()
            .map(|row| u64::from(row.iter().sum::<u32>()))
            .sum();
        sum / 2
    }
}

/// Computes P(i beats j) given log-strength parameters.
///
/// Uses the Bradley-Terry formula: `1 / (1 + exp(-(sᵢ - sⱼ)))`
/// where sᵢ and sⱼ are log-strengths (as stored in [`PhotoRating::strength`]).
///
/// Returns 0.5 when strengths are equal, approaches 1.0 as i's advantage grows.
#[must_use]
pub fn win_probability(strength_i: f64, strength_j: f64) -> f64 {
    1.0 / (1.0 + (-(strength_i - strength_j)).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitive_ranking() {
        let mut bt = BradleyTerry::new(3).unwrap();
        bt.record_comparison(0, 1);
        bt.record_comparison(0, 2);
        bt.record_comparison(1, 2);

        let ratings = bt.compute_ratings(50);
        assert_eq!(ratings[0].photo_idx, 0);
        assert_eq!(ratings[1].photo_idx, 1);
        assert_eq!(ratings[2].photo_idx, 2);
    }

    #[test]
    fn equal_strengths_give_even_odds() {
        let prob = win_probability(0.0, 0.0);
        assert!((prob - 0.5).abs() < 0.001);
    }

    #[test]
    fn higher_strength_wins_more() {
        assert!(win_probability(1.0, 0.0) > 0.5);
        assert!(win_probability(0.0, 1.0) < 0.5);
    }

    #[test]
    fn uncertainty_decreases_with_comparisons() {
        let mut bt = BradleyTerry::new(3).unwrap();
        let initial = bt.compute_uncertainty(0);

        bt.record_comparison(0, 1);
        bt.record_comparison(0, 2);

        assert!(bt.compute_uncertainty(0) < initial);
    }
}
