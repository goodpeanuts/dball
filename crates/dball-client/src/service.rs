use std::collections::HashMap;

use dball_combora::dball::DBall;

use crate::{
    db::{
        spot::{get_all_unprize_spots, update_spot_prize_status_by_id},
        tickets::{get_tickets_by_period, insert_ticket},
    },
    models::Ticket,
    request::latest_ticket::get_latest_lottery,
};

pub async fn update_latest_ticket() -> anyhow::Result<()> {
    let request_latest_ticket = get_latest_lottery()
        .await?
        .data
        .and_then(|t| Ticket::try_from(t).ok())
        .ok_or_else(|| {
            let message = "Failed to get latest ticket".to_owned();
            log::error!("{message}");
            anyhow::anyhow!("{message}")
        })?;

    let query_tickets = get_tickets_by_period(&request_latest_ticket.period)?;

    match query_tickets.len().cmp(&1) {
        std::cmp::Ordering::Equal => {
            let query_ticket = &query_tickets[0];
            if query_ticket == &request_latest_ticket {
                log::debug!("Latest ticket is up to date");
                Ok(())
            } else {
                let msg = format!(
                    "Latest ticket is not match: \n query_ticket: {query_ticket} \n latest_ticket: {request_latest_ticket}"
                );
                log::error!("{msg}");
                Err(anyhow::anyhow!("{msg}"))
            }
        }
        std::cmp::Ordering::Less => {
            log::debug!("Latest ticket not exist, insert");
            insert_ticket(&request_latest_ticket)?;
            Ok(())
        }
        std::cmp::Ordering::Greater => {
            let msg = format!(
                "Multiple tickets found for period {}",
                request_latest_ticket.period
            );
            let tickets_str = query_tickets
                .iter()
                .map(|t| format!("{t}"))
                .collect::<Vec<_>>()
                .join("\n");
            log::error!("{msg}\n{tickets_str}");
            Err(anyhow::anyhow!("{msg}"))
        }
    }
}

pub async fn update_all_spots() -> anyhow::Result<()> {
    let spots = get_all_unprize_spots()?;

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
            let query_tickets = get_tickets_by_period(&spot_period)?;
            let opened_dball = if query_tickets.len().eq(&1) {
                &query_tickets[0].to_dball()?
            } else {
                return Err(anyhow::anyhow!(
                    "Expected exactly one ticket for period {}, found {}",
                    spot_period,
                    query_tickets.len()
                ));
            };

            let reward_price = dball_to_check.1.check_prize(opened_dball).to_i32();

            match update_spot_prize_status_by_id(dball_to_check.0, Some(reward_price)) {
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
