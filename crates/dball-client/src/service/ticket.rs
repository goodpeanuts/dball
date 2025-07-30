use crate::models::Ticket;
use chrono::Datelike as _;
const YEAR_MODULO: usize = 100;

pub async fn crawl_all_tickets() -> anyhow::Result<()> {
    const YEARS: [usize; 23] = [
        2003, 2004, 2005, 2006, 2007, 2008, 2009, 2010, 2011, 2012, 2013, 2014, 2015, 2016, 2017,
        2018, 2019, 2020, 2021, 2022, 2023, 2024, 2025,
    ];
    for &year in YEARS.iter().rev() {
        log::info!("crawl year {year}");
        update_tickets_with_year(year).await?;
    }
    Ok(())
}

pub async fn update_this_year_ticket() -> anyhow::Result<()> {
    let year = chrono::Utc::now().year() as usize;
    update_tickets_with_year(year).await?;
    Ok(())
}

pub async fn update_tickets_with_year(year: usize) -> anyhow::Result<()> {
    // Get existing periods for this year from database
    let existing_periods_7digit = get_existing_periods_for_year(year)?;

    if let Some(latest_period) = existing_periods_7digit.last() {
        log::info!(
            "Found {} existing periods for year {year}",
            existing_periods_7digit.len()
        );

        // Fill gaps in existing data
        update_missing_periods(&existing_periods_7digit).await?;

        // Continue from the latest period
        let latest_period = *latest_period;
        log::info!("Latest period: {latest_period}");

        update_tickets_after_period(latest_period + 1).await?;
    } else {
        log::info!("No existing data for year {year}, starting from period 001");
        update_year_from_start(year).await?;
    }

    Ok(())
}

pub async fn update_latest_ticket() -> anyhow::Result<String> {
    use crate::api::MXNZP_PROVIDER;
    use crate::db::tickets;

    let request_latest_ticket = MXNZP_PROVIDER
        .get_latest_lottery()
        .await?
        .data
        .and_then(|t| Ticket::try_from(t).ok())
        .ok_or_else(|| anyhow::anyhow!("Failed to get latest ticket from API"))?;

    let query_tickets = tickets::get_ticket_by_period(&request_latest_ticket.period)?;

    if let Some(query_ticket) = query_tickets {
        if query_ticket == request_latest_ticket {
            log::info!("Latest ticket is up to date");
            Ok(request_latest_ticket.get_5_digit_period())
        } else {
            anyhow::bail!(
                "Latest ticket mismatch - database: {}, API: {}",
                query_ticket,
                request_latest_ticket
            );
        }
    } else {
        tickets::insert_ticket(&request_latest_ticket)?;
        log::info!(
            "Latest ticket {} updated successfully",
            request_latest_ticket.period
        );
        Ok(request_latest_ticket.get_5_digit_period())
    }
}

/// Update tickets table by period
/// Return `true` if ticket is inserted, `false` if ticket is up to date
/// period is made up of 2-digit year and 3-digit number, e.g. 23001, 23002, 23003, ...
pub async fn update_tickets_by_period(period: &str) -> anyhow::Result<bool> {
    use crate::api::MXNZP_PROVIDER;
    use crate::api::ProviderResponse as _;
    use crate::db::tickets;

    // Check if period is longer than 5 digits and truncate if necessary
    let period = if period.len() > 5 {
        let period_5digit = &period[period.len() - 5..];
        log::warn!(
            "Period {period} is longer than 5 digits, truncating to last 5 digits {period_5digit}"
        );
        period_5digit
    } else {
        period
    };

    if period.len() != 5 {
        anyhow::bail!("MXNZP api request param period must be 5 characters long {period}");
    }

    let request_ticket = MXNZP_PROVIDER
        .get_specified_lottery(period)
        .await?
        .get_data()
        .and_then(|t| Ticket::try_from(t).ok())
        .ok_or_else(|| anyhow::anyhow!("Failed to get ticket for period {period} from API"))?;

    if !check_ticket_in_log_db(period, &request_ticket).await? {
        anyhow::bail!("Ticket for period {period} does not match in log database");
    }

    if let Some(t) = tickets::get_ticket_by_period(period)? {
        if t == request_ticket {
            log::debug!("Ticket for period {period} is up to date");
            Ok(false)
        } else {
            anyhow::bail!(
                "Ticket mismatch for period {period} - database: {t}, API: {request_ticket}"
            );
        }
    } else {
        log::info!("Inserting new ticket for period {period}");
        tickets::insert_ticket(&request_ticket)?;
        log::info!("Ticket for period {period} inserted successfully");
        Ok(true)
    }
}

/// Check if the ticket exists in the log database
/// Returns `true` if the ticket matches the log database or not found
/// Returns `false` if the ticket does not match
pub async fn check_ticket_in_log_db(period: &str, ticket: &Ticket) -> anyhow::Result<bool> {
    use crate::db::ticket_log;

    let ticket_log = ticket_log::get_record_by_code(period)?;

    let Some(ticket_log) = ticket_log else {
        log::debug!("No ticket_log found for ticket with period {period}");
        return Ok(true);
    };

    Ok(ticket.to_dball()? == ticket_log.to_dball()?)
}

/// Get existing period numbers for a specific year from the database
/// Returns a sorted vector of period numbers (e.g., [1, 2, 3, 5, 7] for missing 4 and 6)
fn get_existing_periods_for_year(year: usize) -> anyhow::Result<Vec<usize>> {
    use crate::db::tickets;

    let tickets = tickets::get_all_tickets()?;

    let mut periods_7digit: Vec<usize> = tickets
        .iter()
        .filter_map(|ticket| {
            if ticket.period.starts_with(&year.to_string()) && ticket.period.len() == 7 {
                ticket.period.parse::<usize>().ok()
            } else {
                None
            }
        })
        .collect();

    periods_7digit.sort_unstable();
    periods_7digit.dedup();

    log::debug!(
        "Found {} existing periods for year {year}: {:?}",
        periods_7digit.len(),
        periods_7digit
    );
    Ok(periods_7digit)
}

/// Update tickets for a year starting from period 1
async fn update_year_from_start(year: usize) -> anyhow::Result<()> {
    let start_period = year % YEAR_MODULO * 1000 + 1;
    update_tickets_after_period(start_period).await
}

/// Update tickets for a year starting from a specific period number
async fn update_tickets_after_period(start_period_5digit: usize) -> anyhow::Result<()> {
    let mut period_num = start_period_5digit;
    let mut consecutive_failures = 0;
    const MAX_CONSECUTIVE_FAILURES: usize = 3;

    loop {
        let period = period_num.to_string();

        match update_tickets_by_period(&period).await {
            Ok(_) => (),
            Err(e) => {
                log::warn!("Failed to update period {period}: {e}");
                consecutive_failures += 1;

                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    let year = period_num / 1000;
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
async fn update_missing_periods(existing_periods_7digit: &[usize]) -> anyhow::Result<()> {
    if existing_periods_7digit.is_empty() {
        return Ok(());
    }

    let min_period = *existing_periods_7digit
        .iter()
        .min()
        .ok_or_else(|| anyhow::anyhow!("existing_periods should not be empty at this point"))?;
    let max_period = *existing_periods_7digit
        .iter()
        .max()
        .ok_or_else(|| anyhow::anyhow!("existing_periods should not be empty at this point"))?;

    log::debug!("Filling gaps between periods {min_period} and {max_period}");

    for period_num in min_period..=max_period {
        if !existing_periods_7digit.contains(&period_num) {
            let period = (period_num % 100000).to_string();
            log::info!("Attempting to fill missing period: {period}");

            match update_tickets_by_period(&period).await {
                Ok(inserted) => {
                    if inserted {
                        log::info!("Successfully filled missing period {period}");
                    } else {
                        log::warn!("Period {period} already exists (race condition?)");
                    }
                }
                Err(e) => {
                    log::warn!("Failed to fill missing period {period}: {e}");
                }
            }
        }
    }

    Ok(())
}
