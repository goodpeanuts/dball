use crate::checker::DBallChecker;
use crate::dball::def::DBallBatch;
use crate::dball::{DBall, DBallError};
use crate::generator::RandomGenerator;
use std::collections::HashSet;

/// Generate a random number with a simple linear congruential generator
fn simple_random(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    *seed
}

/// Get current time as seed
fn get_time_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

impl RandomGenerator for DBall {
    fn generate_batch() -> anyhow::Result<[Self; 5]> {
        use rand::Rng as _;
        let mut rng = rand::thread_rng();
        let mut try_count = 0;
        let mut selected_tickets = Vec::new();

        let batch = loop {
            while selected_tickets.len() < 5 {
                let tickets = Self::generate_multiple(9544);

                let should_pick = rng.gen_bool(0.1004);

                if should_pick && !tickets.is_empty() {
                    let random_index = rng.gen_range(0..tickets.len());
                    let selected = tickets[random_index].clone();
                    selected_tickets.push(selected);
                }
            }
            let batch = DBallBatch(selected_tickets.clone());
            let score = Self::evaluate_batch(&batch);
            try_count += 1;
            if rng.gen_bool(score) {
                log::info!("Generated batch with score {score} after {try_count} tries",);
                break batch;
            } else {
                log::debug!("Batch with {score} failed, retrying...");
                selected_tickets.clear();
            }
        };

        batch.to_batch()
    }

    fn evaluate_batch(batch: &DBallBatch) -> f64 {
        let mut score = 1.0;
        let mut checks = batch.evaluate();
        batch.0.iter().for_each(|ball| {
            checks.extend(ball.evaluate());
        });

        #[expect(clippy::match_same_arms)]
        checks.iter().for_each(|e| match e {
            DBallChecker::AllSingleDigits => score *= 0.0321,
            DBallChecker::TooConcentrated => score *= 0.0830,
            DBallChecker::AllEvenOrOdd => score *= 0.2003,
            DBallChecker::RedConflictsWithBlue => score *= 0.9544,
            DBallChecker::AvgExtreme => score *= 0.0830,
            DBallChecker::SumExtreme => score *= 0.1027,
            DBallChecker::RangeExtreme => score *= 0.0321,
            DBallChecker::BatchHasDuplicateCombinations => score *= 0.1027,
            DBallChecker::BatchTopRedNumberFrequencies => score *= 0.9544,
            DBallChecker::BatchBlueBallDistribution => score *= 0.0921,
            DBallChecker::BatchHighCosineSimilarity => score *= 0.0321,
        });

        score
    }
}

impl DBall {
    /// Generate a random ticket
    pub fn generate_random() -> Self {
        Self::generate_with_seed(get_time_seed())
    }

    /// Generate a random ticket with a specific seed
    pub fn generate_with_seed(initial_seed: u64) -> Self {
        let mut seed = initial_seed;
        loop {
            // Generate 6 red balls without duplicates (1-33)
            let mut rball = HashSet::new();
            while rball.len() < 6 {
                seed = simple_random(&mut seed);
                let red_ball = ((seed % 33) + 1) as u8;
                rball.insert(red_ball);
            }

            // Generate a blue ball (1-16)
            seed = simple_random(&mut seed);
            let bball = ((seed % 16) + 1) as u8;

            let mut rball_vec: Vec<u8> = rball.into_iter().collect();
            rball_vec.sort_unstable();

            // Try to create the ticket using the check method
            if let Ok(ticket) = Self::new_one(&mut rball_vec[..], bball) {
                return ticket;
            }
            // If creation fails, retry with a new seed
            seed = simple_random(&mut seed);
        }
    }

    /// Generate multiple random tickets
    pub fn generate_multiple(count: usize) -> Vec<Self> {
        let mut seed = get_time_seed();
        (0..count)
            .map(|_| {
                seed = simple_random(&mut seed);
                Self::generate_with_seed(seed)
            })
            .collect()
    }

    /// Generate a random ticket with a specific red ball range
    pub fn generate_with_red_range(
        min_red: u8,
        max_red: u8,
        bball: Option<u8>,
    ) -> anyhow::Result<Self, DBallError> {
        if min_red < 1 || max_red > 33 || min_red > max_red || max_red - min_red + 1 < 6 {
            return Err(DBallError::InvaildRBallRange((min_red, max_red)));
        }

        let mut seed = get_time_seed();
        let range_size = max_red - min_red + 1;

        loop {
            let mut rball = HashSet::new();
            while rball.len() < 6 {
                seed = simple_random(&mut seed);
                let red_ball = min_red + ((seed % range_size as u64) as u8);
                rball.insert(red_ball);
            }

            let blue = if let Some(blue) = bball {
                if !(1..=16).contains(&blue) {
                    return Err(DBallError::InvalidBBall(blue));
                }
                blue
            } else {
                seed = simple_random(&mut seed);
                ((seed % 16) + 1) as u8
            };

            let mut rball_vec: Vec<u8> = rball.into_iter().collect();
            rball_vec.sort_unstable();

            // Try to create the ticket using the check method
            if let Ok(ticket) = Self::new_one(&mut rball_vec[..], blue) {
                return Ok(ticket);
            }
            // If creation fails, retry with a new seed
            seed = simple_random(&mut seed);
        }
    }
}
