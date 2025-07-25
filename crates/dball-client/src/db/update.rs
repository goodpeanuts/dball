use crate::{db::tickets::insert_ticket, models::Ticket, request::mxnzp::get_latest_lottery};

use super::tickets::get_tickets_by_period;

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
