use crate::dball::{DBall, DBallBatch};
use std::collections::{HashMap, HashSet};

pub enum DBallChecker {
    AllSingleDigits,
    AllEvenOrOdd,
    RedConflictsWithBlue,
    AvgExtreme,
    SumExtreme,
    RangeExtreme,
    BatchHasDuplicateCombinations,
    BatchTopRedNumberFrequencies,
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

    pub fn avg_extreme(&self) -> Option<DBallChecker> {
        let avg = self.rball.iter().copied().sum::<u8>() / 5;
        (avg < 10 || avg > 21).then_some(DBallChecker::AvgExtreme)
    }

    pub fn is_range_extreme(&self) -> Option<DBallChecker> {
        const MIN_GAP: u8 = 8;
        const MAX_GAP: u8 = 25;
        let min = *self.rball.iter().min().unwrap_or(&0);
        let max = *self.rball.iter().max().unwrap_or(&33);
        let gap = max - min;
        (gap < MIN_GAP || gap > MAX_GAP).then_some(DBallChecker::RangeExtreme)
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
            (count_first - count_last)
                .ge(&3)
                .then_some(DBallChecker::BatchTopRedNumberFrequencies)
        } else {
            None
        }
    }

    pub fn blue_ball_distribution(&self) -> Option<DBallChecker> {
        let mut b = 0;
        let mut cnt = 0;
        for ball in &self.0 {
            if cnt == 0 && b != ball.bball {
                b = ball.bball;
                cnt += 1;
            } else {
                cnt -= 1;
            }
        }
        cnt.gt(&2)
            .then_some(DBallChecker::BatchBlueBallDistribution)
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
        if let Some(check) = self.has_high_cosine_similarity() {
            checks.push(check);
        }
        checks
    }
}
