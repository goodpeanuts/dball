#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use dball_client::request::{GENERAL_LATEST_LOTTERY_REQUEST, SendRequest};

    let resp = GENERAL_LATEST_LOTTERY_REQUEST.send().await?;

    println!("Response status: {}", resp.status());

    Ok(())
}
