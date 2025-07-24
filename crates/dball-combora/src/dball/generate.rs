use crate::dball::{DBall, DBallError};
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
