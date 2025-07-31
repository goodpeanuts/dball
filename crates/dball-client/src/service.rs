mod spot;
mod ticket;

pub use spot::{insert_new_spots_next_period, update_all_unprize_spots};
pub use ticket::{
    check_ticket_in_log_db, crawl_all_tickets, get_next_period, update_latest_ticket,
    update_tickets_by_period, update_tickets_with_year,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_latest_ticket() {
        match update_latest_ticket().await {
            Ok(_) => log::info!("Latest ticket updated successfully"),
            Err(e) => {
                panic!("Failed to update latest ticket: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_update_all_spots() {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        match update_latest_ticket().await {
            Ok(_) => log::info!("Latest ticket updated successfully"),
            Err(e) => {
                panic!("Failed to update latest ticket: {e}");
            }
        }

        match update_all_unprize_spots().await {
            Ok(_) => log::info!("All spots updated successfully"),
            Err(e) => {
                log::error!("Failed to update all spots: {e}");
                panic!("Failed to update all spots: {e}")
            }
        }
    }
}
