use crate::dball::{DBall, DBallBatch};
use std::collections::{HashMap, HashSet};

pub enum DBallChecker {
    AllSingleDigits,
    AllEvenOrOdd,
    RedConflictsWithBlue,
    SumExtreme,
    RangeExtreme,
    BatchRBallSumExtreme,
    BatchHasDuplicateCombinations,
    BatchTopRedNumberFrequencies,
    BatchBlueBallDuplicate,
    BatchBlueBallDistribution,
    BatchHighCosineSimilarity,
}

impl DBall {
    pub fn is_all_single_digits(&self) -> Option<DBallChecker> {
        self.rball
            .iter()
            .all(|&n| n < 10)
            .then_some(DBallChecker::AllSingleDigits)
    }

    pub fn is_all_even_or_odd(&self) -> Option<DBallChecker> {
        (self.rball.iter().all(|&n| n % 2 == 0) || self.rball.iter().all(|&n| n % 2 == 1))
            .then_some(DBallChecker::AllEvenOrOdd)
    }

    pub fn red_conflicts_with_blue(&self) -> Option<DBallChecker> {
        self.rball
            .contains(&self.bball)
            .then_some(DBallChecker::RedConflictsWithBlue)
    }

    pub fn sum_extreme(&self) -> Option<DBallChecker> {
        const SUM_EXTREME_MIN: u8 = 13 * 5;
        const SUM_EXTREME_MAX: u8 = 19 * 5;
        let sum = self.rball.iter().copied().sum::<u8>();
        (!(SUM_EXTREME_MIN..=SUM_EXTREME_MAX).contains(&sum)).then_some(DBallChecker::SumExtreme)
    }

    pub fn is_range_extreme(&self) -> Option<DBallChecker> {
        const MIN_GAP: u8 = 8;
        const MAX_GAP: u8 = 25;
        let min = *self.rball.iter().min().unwrap_or(&0);
        let max = *self.rball.iter().max().unwrap_or(&33);
        let gap = max - min;
        (!((MIN_GAP)..=(MAX_GAP)).contains(&gap)).then_some(DBallChecker::RangeExtreme)
    }

    pub fn evaluate(&self) -> Vec<DBallChecker> {
        let mut checks = Vec::new();
        if let Some(check) = self.is_all_single_digits() {
            checks.push(check);
        }
        if let Some(check) = self.is_all_even_or_odd() {
            checks.push(check);
        }
        if let Some(check) = self.red_conflicts_with_blue() {
            checks.push(check);
        }
        if let Some(check) = self.is_range_extreme() {
            checks.push(check);
        }
        checks
    }
}

impl DBallBatch {
    pub fn batch_sum_extreme(&self) -> Option<DBallChecker> {
        const SUM_EXTREME_MIN: usize = 13 * 5 * 5;
        const SUM_EXTREME_MAX: usize = 19 * 5 * 5;
        let sum = self
            .0
            .iter()
            .map(|b| b.rball.iter().map(|n| *n as usize).sum::<usize>())
            .sum::<usize>();
        (!(SUM_EXTREME_MIN..=SUM_EXTREME_MAX).contains(&sum))
            .then_some(DBallChecker::BatchRBallSumExtreme)
    }

    pub fn has_duplicate_combinations(&self) -> Option<DBallChecker> {
        let mut seen = HashSet::new();
        for ball in &self.0 {
            let mut key = ball.rball.to_vec();
            key.sort_unstable();
            key.push(ball.bball); // Include blue ball in combination uniqueness
            if !seen.insert(key) {
                return Some(DBallChecker::BatchHasDuplicateCombinations);
            }
        }
        None
    }

    pub fn top_red_number_frequencies(&self, top_n: usize) -> Option<DBallChecker> {
        let mut freq = HashMap::new();
        for ball in &self.0 {
            for &n in &ball.rball {
                *freq.entry(n).or_insert(0) += 1;
            }
        }
        let mut freq_vec: Vec<_> = freq.into_iter().collect();
        freq_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let vec = freq_vec
            .into_iter()
            .take(top_n)
            .collect::<Vec<(u8, usize)>>();
        if let (Some((_, count_first)), Some((_, count_last))) = (vec.first(), vec.last()) {
            ((count_first - count_last).ge(&3) || (count_first.gt(&2)))
                .then_some(DBallChecker::BatchTopRedNumberFrequencies)
        } else {
            None
        }
    }

    pub fn blue_ball_distribution(&self) -> Option<DBallChecker> {
        let avg = self.0.iter().map(|b| b.bball).sum::<u8>() as f64 / self.0.len() as f64;
        if !(6.0..=10.0).contains(&avg) {
            return Some(DBallChecker::BatchBlueBallDistribution);
        }
        None
    }

    pub fn duplicate_bball(&self) -> Option<DBallChecker> {
        let mut bball_count = HashMap::new();
        for ball in &self.0 {
            *bball_count.entry(ball.bball).or_insert(0) += 1;
        }
        if bball_count.values().any(|&count| count > 1) {
            Some(DBallChecker::BatchBlueBallDuplicate)
        } else {
            None
        }
    }

    pub fn has_high_cosine_similarity(&self) -> Option<DBallChecker> {
        let sims = self.cosine_similarity();

        (sims.iter().filter(|&&t| t == 0.0).count().le(&4) || sims.iter().any(|&sim| sim > 0.3))
            .then_some(DBallChecker::BatchHighCosineSimilarity)
    }

    pub fn evaluate(&self) -> Vec<DBallChecker> {
        let mut checks = Vec::new();
        if let Some(check) = self.has_duplicate_combinations() {
            checks.push(check);
        }
        if let Some(check) = self.top_red_number_frequencies(5) {
            checks.push(check);
        }
        if let Some(check) = self.blue_ball_distribution() {
            checks.push(check);
        }
        if let Some(check) = self.duplicate_bball() {
            checks.push(check);
        }
        if let Some(check) = self.has_high_cosine_similarity() {
            checks.push(check);
        }
        checks
    }
}
