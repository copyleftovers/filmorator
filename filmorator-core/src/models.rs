use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Photo {
    pub id: Uuid,
    pub filename: String,
    pub file_hash: String,
    pub position: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

impl Session {
    #[must_use]
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            last_active_at: now,
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Matchup {
    pub id: Uuid,
    pub session_id: Uuid,
    pub photo_indices: Vec<u32>,
    pub is_seed: bool,
    pub created_at: DateTime<Utc>,
}

impl Matchup {
    #[must_use]
    pub fn new(session_id: Uuid, photo_indices: Vec<u32>, is_seed: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            photo_indices,
            is_seed,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub id: Uuid,
    pub matchup_id: Uuid,
    pub session_id: Uuid,
    pub ranked_photo_indices: Vec<u32>,
    pub created_at: DateTime<Utc>,
}

impl ComparisonResult {
    #[must_use]
    pub fn new(matchup_id: Uuid, session_id: Uuid, ranked_photo_indices: Vec<u32>) -> Self {
        Self {
            id: Uuid::new_v4(),
            matchup_id,
            session_id,
            ranked_photo_indices,
            created_at: Utc::now(),
        }
    }

    #[must_use]
    pub fn to_pairwise(&self) -> Vec<(u32, u32)> {
        let mut pairs = Vec::new();
        for (i, &winner) in self.ranked_photo_indices.iter().enumerate() {
            for &loser in &self.ranked_photo_indices[i + 1..] {
                pairs.push((winner, loser));
            }
        }
        pairs
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PhotoRating {
    pub photo_idx: u32,
    pub strength: f64,
    pub uncertainty: f64,
}

impl PhotoRating {
    #[must_use]
    pub fn new(photo_idx: u32) -> Self {
        Self {
            photo_idx,
            strength: 0.0,
            uncertainty: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_result_to_pairwise() {
        let result = ComparisonResult::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            vec![3, 1, 2], // 3 > 1 > 2
        );
        let pairs = result.to_pairwise();
        assert_eq!(pairs, vec![(3, 1), (3, 2), (1, 2)]);
    }
}
