#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dball_client::setup(Some(log::LevelFilter::Info));
    dball_client::service::crawl_all_tickets().await?;

    Ok(())
}
