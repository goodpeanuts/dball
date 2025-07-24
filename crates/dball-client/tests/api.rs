#[tokio::test]
async fn test_general_latest_lottery() -> anyhow::Result<()> {
    use dball_client::request::{GENERAL_LATEST_LOTTERY_REQUEST, SendRequest};

    let resp = GENERAL_LATEST_LOTTERY_REQUEST.send().await?;

    assert_eq!(resp.status(), 200);

    Ok(())
}
