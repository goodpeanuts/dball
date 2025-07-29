use crate::api::{MXNZP_PROVIDER, ProviderResponse as _};
use crate::db::{spot, tickets};
use crate::models::Ticket;
use crate::service::ticket::update_this_year_ticket;
use dball_combora::dball::DBall;
use std::collections::HashMap;

use super::update_latest_ticket;

pub async fn update_all_unprize_spots() -> anyhow::Result<()> {
    let spots = spot::get_all_unprize_spots()?;

    if spots.is_empty() {
        log::info!("No unprized spots found, nothing to update");
        return Ok(());
    }
    update_this_year_ticket().await?;

    log::debug!("Found {} unprized spots", spots.len());
    let mut spots_by_period: HashMap<String, Vec<(i32, DBall)>> = HashMap::new();
    for spot in spots {
        spots_by_period
            .entry(spot.period.clone())
            .or_default()
            .push((
                spot.id.expect(crate::NEVER_NONE_BY_DATABASE),
                TryFrom::try_from(spot)?,
            ));
    }

    let mut errors = Vec::new();
    #[expect(clippy::iter_over_hash_type)]
    for (spot_period, dballs_to_check) in spots_by_period {
        log::debug!(
            "Processing {} spots for period {spot_period}",
            dballs_to_check.len()
        );

        let opened_ball = if let Some(t) = tickets::get_ticket_by_period(&spot_period)? {
            t.to_dball()?
        } else {
            log::warn!("No ticket found for period {spot_period}, fetching latest ticket");
            let ticket = MXNZP_PROVIDER
                .get_latest_lottery()
                .await?
                .get_data()
                .and_then(|t| Ticket::try_from(t).ok())
                .ok_or_else(|| {
                    anyhow::anyhow!("Failed to get latest ticket for period {}", spot_period)
                })?;
            ticket.to_dball()?
        };

        // update the spot by checking with the opened dball
        for dball_to_check in dballs_to_check {
            let reward_price = dball_to_check.1.check_prize(&opened_ball).to_i32();

            match spot::update_spot_prize_status_by_id(dball_to_check.0, Some(reward_price)) {
                Ok(()) => {
                    log::debug!(
                        "Updated spot for id {id} with reward level {reward_price}",
                        id = dball_to_check.0
                    );
                }
                Err(e) => {
                    errors.push(e.to_string());
                }
            }
        }
    }

    if !errors.is_empty() {
        let e = errors.join("\n");
        anyhow::bail!("Failed to update some spots:\n{e}");
    }

    log::info!("Completed updating all spots");
    Ok(())
}

pub async fn insert_new_spots_next_period(dballs: &[DBall]) -> anyhow::Result<()> {
    let next_period = update_latest_ticket().await?;
    for dball in dballs {
        spot::insert_spot_from_dball(&next_period, dball, None)?;
    }
    Ok(())
}

#[tokio::test]
async fn insert_dball_batch() -> anyhow::Result<()> {
    // dball_client::setup(Some(log::LevelFilter::Info));

    log::info!("Running in terminal mode with concurrent batch generation (10 threads).");
    use dball_combora::dball::DBallBatch;
    use dball_combora::generator::RandomGenerator as _;
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex};
    use std::thread;

    const THREAD_COUNT: usize = 8;

    // Create a channel to receive results from threads
    let (tx, rx) = mpsc::channel();
    let result_found = Arc::new(Mutex::new(false));

    // Spawn 10 threads to generate batches concurrently
    for i in 0..THREAD_COUNT {
        let tx_clone = tx.clone();
        let result_found_clone = Arc::clone(&result_found);

        thread::spawn(move || {
            log::debug!("Thread {} starting batch generation", i);

            // Continuously try to generate batch until one is found or another thread succeeds
            loop {
                // Check if another thread already found a result
                {
                    let found = result_found_clone.lock().unwrap();
                    if *found {
                        log::debug!(
                            "Thread {} exiting - result already found by another thread",
                            i
                        );
                        return;
                    }
                }

                match dball_combora::dball::DBall::generate_batch() {
                    Ok(tickets) => {
                        // Mark that we found a result
                        {
                            let mut found = result_found_clone.lock().unwrap();
                            if !*found {
                                *found = true;
                                log::info!("Thread {} successfully generated batch first!", i);
                                // Send the result
                                let _ = tx_clone.send((i, tickets));
                                return;
                            } else {
                                // Another thread already found a result
                                log::debug!(
                                    "Thread {} generated batch but another thread was first",
                                    i
                                );
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        log::debug!("Thread {} failed to generate batch, retrying: {}", i, e);
                        // Continue the loop to try again
                    }
                }
            }
        });
    }

    // Drop the original sender so rx.recv() can detect when all threads are done
    drop(tx);

    // Wait for the first successful result
    match rx.recv() {
        Ok((thread_id, tickets)) => {
            log::info!("First successful batch generated by thread {}", thread_id);
            log::info!("Generated tickets:\n{}", DBallBatch(tickets.to_vec()));

            // Signal other threads to stop
            {
                let mut found = result_found.lock().unwrap();
                *found = true;
            }

            log::info!(
                "Batch generation completed successfully. Other threads will stop automatically."
            );
            insert_new_spots_next_period(&tickets).await?;
            Ok(())
        }
        Err(_) => {
            // All threads finished without success
            anyhow::bail!("All batch generation attempts failed")
        }
    }
}
