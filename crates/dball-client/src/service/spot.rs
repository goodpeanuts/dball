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

pub async fn insert_new_spots_next_period(dballs: Vec<DBall>) -> anyhow::Result<()> {
    let next_period = update_latest_ticket().await?;
    for dball in dballs {
        spot::insert_spot_from_dball(&next_period, &dball, None)?;
    }
    Ok(())
}

// #[cfg(test)]
// mod test {
//     use super::*;

// }
