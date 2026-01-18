use std::sync::atomic::AtomicBool;

use super::{DBall, DBallBatch, DBallChecker, DBallError, HashSet, RandomGenerator};
pub struct BlueMorn;

impl RandomGenerator for BlueMorn {
    fn generate_batch(&self) -> anyhow::Result<[DBall; 5]> {
        const THREAD_COUNT: usize = 10;
        let batch = self.multi_thread_generate(THREAD_COUNT)?;
        batch.to_batch()
    }

    fn evaluate_batch(&self, batch: &DBallBatch) -> f64 {
        let mut score = 1.0;
        let mut checks = batch.evaluate();
        batch.0.iter().for_each(|ball| {
            checks.extend(ball.evaluate());
        });

        #[expect(clippy::match_same_arms)]
        for e in &checks {
            match e {
                DBallChecker::AllSingleDigits => score *= 0.1004,
                DBallChecker::AllEvenOrOdd => score *= 0.2003,
                DBallChecker::RedConflictsWithBlue => score *= 0.0921,
                DBallChecker::SumExtreme => score *= 0.1027,
                DBallChecker::RangeExtreme => score *= 0.3544,
                DBallChecker::BatchRBallSumExtreme => score *= 0.3544,
                DBallChecker::BatchHasDuplicateCombinations => score *= 0.0321,
                DBallChecker::BatchTopRedNumberFrequencies => score *= 0.0321,
                DBallChecker::BatchBlueBallDistribution => score *= 0.0921,
                DBallChecker::BatchBlueBallDuplicate => score *= 0.0321,
                DBallChecker::BatchHighCosineSimilarity => score *= 0.0830,
            }
        }
        score
    }
}

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

impl BlueMorn {
    /// Generate a random ticket
    pub fn generate_random() -> DBall {
        Self::generate_with_seed(get_time_seed())
    }

    /// Generate a random ticket with a specific seed
    pub fn generate_with_seed(initial_seed: u64) -> DBall {
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
            if let Ok(ticket) = DBall::new_one(&mut rball_vec[..], bball) {
                return ticket;
            }
            // If creation fails, retry with a new seed
            seed = simple_random(&mut seed);
        }
    }

    /// Generate multiple random tickets
    pub fn generate_multiple(&self, count: usize) -> Vec<DBall> {
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
    ) -> anyhow::Result<DBall, DBallError> {
        if min_red < 1 || max_red > 33 || min_red > max_red || max_red - min_red + 1 < 6 {
            return Err(DBallError::InvalidRBallRange((min_red, max_red)));
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
            if let Ok(ticket) = DBall::new_one(&mut rball_vec[..], blue) {
                return Ok(ticket);
            }
            // If creation fails, retry with a new seed
            seed = simple_random(&mut seed);
        }
    }

    fn generate_dball_batch(&self, stop: &AtomicBool) -> Option<DBallBatch> {
        const ITER_CHECK: usize = 0xFF;
        use rand::Rng as _;
        let mut rng = rand::thread_rng();
        let mut try_count = 0;
        let mut selected_tickets = Vec::new();
        let mut iter: usize = 0;

        loop {
            iter += 1;
            while selected_tickets.len() < 5 {
                let tickets = self.generate_multiple(3544);

                let should_pick = rng.gen_bool(0.1004);

                if should_pick && !tickets.is_empty() {
                    let random_index = rng.gen_range(0..tickets.len());
                    let selected = tickets[random_index];
                    selected_tickets.push(selected);
                }
            }
            let batch = DBallBatch(selected_tickets.clone());
            let score = self.evaluate_batch(&batch);
            try_count += 1;
            if rng.gen_bool(score) {
                if stop.load(std::sync::atomic::Ordering::Relaxed) {
                    log::debug!("Received stop signal, exiting batch generation");
                    return None;
                }
                log::info!("Generated batch with score {score} after {try_count} tries",);
                return Some(batch);
            } else {
                log::debug!("Batch with {score} failed, retrying...");
                selected_tickets.clear();
            }

            if (iter & ITER_CHECK) == 0 && stop.load(std::sync::atomic::Ordering::Relaxed) {
                log::debug!("Stopping batch generation after {iter} iterations");
                return None;
            } else {
                iter = 0;
            }
        }
    }

    #[expect(clippy::unused_self)]
    fn multi_thread_generate(&self, thread_count: usize) -> anyhow::Result<DBallBatch> {
        use std::sync::{Arc, mpsc};
        use std::thread::{self, JoinHandle};

        // Create a channel to receive results from threads
        let (tx, rx) = mpsc::channel();

        // Store thread handles so we can terminate them
        let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(thread_count);

        let stop = Arc::new(AtomicBool::new(false));

        // Spawn threads to generate batches concurrently
        for i in 0..thread_count {
            let tx_clone = tx.clone();
            let stop_clone = Arc::clone(&stop);

            // No reference to self escapes; use BlueMorn directly
            let handle = thread::spawn(move || {
                log::debug!("Thread {i} starting batch generation");

                // Generate batch (this is a blocking operation until success)
                let generator = Self;
                let tickets = generator.generate_dball_batch(&stop_clone);

                log::info!("Thread {i} successfully generated batch!");
                // Try to send the result - if channel is closed, just exit
                if tx_clone.send((i, tickets)).is_err() {
                    log::debug!("Thread {i} - channel closed, exiting");
                }
            });

            handles.push(handle);
        }

        // Drop the original sender immediately after spawning all threads
        // This ensures rx.recv() can detect when all worker threads are done
        drop(tx);

        // Wait for the first successful result
        if let Ok((thread_id, Some(tickets))) = rx.recv() {
            log::info!("Received result from thread {thread_id}, terminating other threads");
            // Stop all other threads
            stop.store(true, std::sync::atomic::Ordering::Relaxed);

            // Consume and discard any additional results that might come in
            while let Ok((discarded_thread_id, _)) = rx.try_recv() {
                log::debug!(
                    "Discarded result from thread {discarded_thread_id} (already have result)"
                );
            }

            Ok(tickets)
        } else {
            // All threads finished without success this should never happen
            stop.store(true, std::sync::atomic::Ordering::Relaxed);
            anyhow::bail!("batch generation attempts failed");
        }
    }
}
