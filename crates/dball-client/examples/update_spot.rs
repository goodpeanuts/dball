#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dball_client::setup(Some(log::LevelFilter::Info));
    dball_client::service::update_all_unprize_spots().await?;

    Ok(())
}
