use std::collections::HashMap;

use dball_combora::dball::DBall;

/// Used to convert a 4-digit year to a 2-digit format (e.g., 2024 -> 24)
const YEAR_MODULO: usize = 100;

use crate::{
    models::Ticket,
    request::{MXNZP_PROVIDER, provider::ProviderResponse as _},
};

pub async fn update_tickets_table() -> anyhow::Result<()> {
    const YEARS: [usize; 23] = [
        2003, 2004, 2005, 2006, 2007, 2008, 2009, 2010, 2011, 2012, 2013, 2014, 2015, 2016, 2017,
        2018, 2019, 2020, 2021, 2022, 2023, 2024, 2025,
    ];

    for &year in YEARS.iter().rev() {
        log::info!("Processing year {year}");

        // Get existing periods for this year from database
        let existing_periods = get_existing_periods_for_year(year)?;

        if let Some(latest_period) = existing_periods.last() {
            log::info!(
                "Found {} existing periods for year {year}",
                existing_periods.len()
            );

            // Fill gaps in existing data
            let result = fill_missing_periods(year, &existing_periods).await;
            if let Err(e) = result {
                log::error!("Failed to fill missing periods for year {year}: {e}");
            }

            // Continue from the latest period
            let latest_period = *latest_period;
            log::info!("Latest period for year {year}: {latest_period:03}");
            let result = update_year_from_period(year, latest_period + 1).await;
            if let Err(e) = result {
                log::error!(
                    "Failed to continue updating year {year} from period {}: {e}",
                    latest_period + 1
                );
            }
        } else {
            log::info!("No existing data for year {year}, starting from period 001");
            let result = update_year_from_start(year).await;
            if let Err(e) = result {
                log::error!("Failed to update year {year} from start: {e}");
            }
        }
    }

    Ok(())
}

pub async fn update_latest_ticket() -> anyhow::Result<()> {
    use crate::db::tickets;

    let request_latest_ticket = MXNZP_PROVIDER
        .get_latest_lottery()
        .await?
        .data
        .and_then(|t| Ticket::try_from(t).ok())
        .ok_or_else(|| {
            let message = "Failed to get latest ticket".to_owned();
            log::error!("{message}");
            anyhow::anyhow!("{message}")
        })?;

    let query_tickets = tickets::get_ticket_by_period(&request_latest_ticket.period)?;

    if let Some(query_ticket) = query_tickets {
        if query_ticket == request_latest_ticket {
            log::info!("Latest ticket is up to date");
            Ok(())
        } else {
            let msg = format!(
                "Latest ticket is not match: \n query_ticket: {query_ticket} \n latest_ticket: {request_latest_ticket}"
            );
            log::error!("{msg}");
            Err(anyhow::anyhow!("{msg}"))
        }
    } else {
        log::debug!("Inserting latest ticket into database");
        tickets::insert_ticket(&request_latest_ticket)?;
        log::info!("Latest ticket inserted successfully");
        Ok(())
    }
}

/// Update tickets table by period
/// Return `true` if ticket is inserted, `false` if ticket is up to date
pub async fn update_tickets_by_period(period: &str) -> anyhow::Result<bool> {
    use crate::db::tickets;

    let request_ticket = MXNZP_PROVIDER
        .get_specified_lottery(period)
        .await?
        .get_data()
        .and_then(|t| Ticket::try_from(t).ok())
        .ok_or_else(|| {
            let message = format!("Failed to get ticket for period {period}");
            log::error!("{message}");
            anyhow::anyhow!("{message}")
        })?;

    if !check_ticket_in_log_db(period, &request_ticket).await? {
        return Err(anyhow::anyhow!(
            "Ticket for period {period} does not match in log database"
        ));
    }

    if let Some(t) = tickets::get_ticket_by_period(period)? {
        if t == request_ticket {
            log::info!("Ticket for period {period} is up to date");
            Ok(false)
        } else {
            Err(anyhow::anyhow!(
                "Ticket for period {period} does not match in tickets table\nquery_ticket: {t}\nrequest_ticket: {request_ticket}"
            ))
        }
    } else {
        log::debug!("Inserting ticket for period {period}");
        tickets::insert_ticket(&request_ticket)?;
        log::info!("Ticket for period {period} inserted successfully");
        Ok(true)
    }
}

pub async fn check_ticket_in_log_db(period: &str, ticket: &Ticket) -> anyhow::Result<bool> {
    use crate::db::ticket_log;

    let ticket_log = ticket_log::get_record_by_code(period)?;

    let Some(ticket_log) = ticket_log else {
        log::debug!("No ticket_log found for ticket with period {period}");
        return Ok(true);
    };

    Ok(ticket.to_dball()? == ticket_log.to_dball()?)
}

pub async fn update_all_spots() -> anyhow::Result<()> {
    use crate::db::spot;
    use crate::db::tickets;

    let spots = spot::get_all_unprize_spots()?;

    if spots.is_empty() {
        log::debug!("No unprized spots found");
        return Ok(());
    }
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

    #[expect(clippy::iter_over_hash_type)]
    for (spot_period, dballs_to_check) in spots_by_period {
        log::debug!(
            "Processing {} spots for period {}",
            dballs_to_check.len(),
            spot_period
        );

        // TODO: If dballs_to_check.is_empty(), implement logic to request the specified lottery for this period and handle the response accordingly.
        for dball_to_check in dballs_to_check {
            let query_tickets = tickets::get_ticket_by_period(&spot_period)?;
            let opened_dball = if let Some(query_ticket) = query_tickets {
                &query_ticket.to_dball()?
            } else {
                return Err(anyhow::anyhow!(
                    "No ticket found for period {}",
                    spot_period
                ));
            };

            let reward_price = dball_to_check.1.check_prize(opened_dball).to_i32();

            match spot::update_spot_prize_status_by_id(dball_to_check.0, Some(reward_price)) {
                Ok(()) => {
                    log::debug!(
                        "Updated spot for id {id} with reward level {reward_price}",
                        id = dball_to_check.0
                    );
                }
                Err(e) => {
                    log::error!(
                        "Failed to update spot for id {id}: {e}",
                        id = dball_to_check.0
                    );
                }
            }
        }
    }

    log::debug!("Completed updating all spots");
    Ok(())
}

/// Get existing period numbers for a specific year from the database
/// Returns a sorted vector of period numbers (e.g., [1, 2, 3, 5, 7] for missing 4 and 6)
fn get_existing_periods_for_year(year: usize) -> anyhow::Result<Vec<usize>> {
    use crate::db::tickets;

    let year_prefix = format!("{:02}", year % YEAR_MODULO);
    let tickets = tickets::get_all_tickets()?;

    let mut periods: Vec<usize> = tickets
        .iter()
        .filter_map(|ticket| {
            if ticket.period.starts_with(&year_prefix) && ticket.period.len() == 5 {
                ticket.period[2..].parse::<usize>().ok()
            } else {
                None
            }
        })
        .collect();

    periods.sort_unstable();
    periods.dedup();

    log::debug!(
        "Found {} existing periods for year {}: {:?}",
        periods.len(),
        year,
        periods
    );
    Ok(periods)
}

/// Update tickets for a year starting from period 1
async fn update_year_from_start(year: usize) -> anyhow::Result<()> {
    update_year_from_period(year, 1).await
}

/// Update tickets for a year starting from a specific period number
async fn update_year_from_period(year: usize, start_period: usize) -> anyhow::Result<()> {
    let mut period_num = start_period;
    let mut consecutive_failures = 0;
    const MAX_CONSECUTIVE_FAILURES: usize = 3;

    loop {
        let period = format!("{:02}{:03}", year % YEAR_MODULO, period_num);

        match update_tickets_by_period(&period).await {
            Ok(_) => {
                consecutive_failures = 0; // Reset failure counter on success
            }
            Err(e) => {
                log::warn!("Failed to update period {period}: {e}");
                consecutive_failures += 1;

                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    log::info!(
                        "Stopping updates for year {year} after {MAX_CONSECUTIVE_FAILURES} consecutive failures"
                    );
                    break;
                }
            }
        }

        period_num += 1;
    }

    Ok(())
}

/// Fill missing periods in the existing data for a specific year
async fn fill_missing_periods(year: usize, existing_periods: &[usize]) -> anyhow::Result<()> {
    if existing_periods.is_empty() {
        return Ok(());
    }

    let min_period = *existing_periods
        .iter()
        .min()
        .ok_or_else(|| anyhow::anyhow!("existing_periods should not be empty at this point"))?;
    let max_period = *existing_periods
        .iter()
        .max()
        .ok_or_else(|| anyhow::anyhow!("existing_periods should not be empty at this point"))?;

    log::info!("Filling gaps between periods {min_period:03} and {max_period:03} for year {year}");

    for period_num in min_period..=max_period {
        if !existing_periods.contains(&period_num) {
            let period = format!("{:02}{:03}", year % YEAR_MODULO, period_num);
            log::info!("Attempting to fill missing period: {period}");

            match update_tickets_by_period(&period).await {
                Ok(inserted) => {
                    if inserted {
                        log::info!("Successfully filled missing period {period}");
                    } else {
                        log::debug!("Period {period} already exists (race condition?)");
                    }
                }
                Err(e) => {
                    log::warn!("Failed to fill missing period {period}: {e}");
                    // Continue trying other missing periods even if one fails
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_latest_ticket() {
        match update_latest_ticket().await {
            Ok(_) => log::info!("Latest ticket updated successfully"),
            Err(e) => panic!("Failed to update latest ticket: {e}"),
        }
    }

    #[tokio::test]
    async fn test_update_all_spots() {
        update_latest_ticket()
            .await
            .expect("Failed to update latest ticket");
        match update_all_spots().await {
            Ok(_) => log::info!("All spots updated successfully"),
            Err(e) => {
                log::error!("Failed to update all spots: {}", e);
                panic!("Failed to update all spots: {e}")
            }
        }
    }
}
