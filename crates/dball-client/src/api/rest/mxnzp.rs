mod common;
mod latest_ticket;
mod specified_ticket;

#[cfg(test)]
mod tests {
    use crate::api::MXNZP_PROVIDER;
    use crate::api::provider::ProviderResponse as _;
    use crate::models::Ticket;

    #[tokio::test]
    async fn test_mxnzp_latest_lottery() {
        let resp = MXNZP_PROVIDER.get_latest_lottery().await;

        if let Ok(response) = resp {
            assert_eq!(response.get_code(), 1);
            let data = response.get_data();
            assert!(data.is_some());

            let data = data.expect("Failed to get data");
            log::debug!("data: {data:#?}");

            // 使用 try_from 进行正确的类型转换
            let ticket = Ticket::try_from(data);
            assert!(ticket.is_ok(), "Failed to convert LotteryData to Ticket");
            if let Ok(ticket) = ticket {
                log::debug!("converted ticket: {ticket:#?}");
            }
        } else if let Err(e) = resp {
            log::warn!(
                "Failed to get lottery data (this is expected if config is not set up): {e}"
            );
        }
    }

    #[tokio::test]
    async fn test_mxnzp_specified_lottery() {
        let expect = "2025084";
        let resp = MXNZP_PROVIDER.get_specified_lottery(expect).await;

        if let Ok(response) = resp {
            if let Some(data) = response.get_data() {
                log::debug!("API Response data: {data:?}");
                let ticket = Ticket::try_from(data);
                assert!(ticket.is_ok(), "Failed to convert LotteryData to Ticket");

                if let Ok(ticket) = ticket {
                    log::debug!("Converted ticket: {ticket}");
                }
            } else {
                panic!("Failed to get specified lottery");
            };
        } else if let Err(e) = resp {
            log::warn!(
                "Failed to get specified lottery data (this is expected if config is not set up): {e}"
            );
        }
    }
}
