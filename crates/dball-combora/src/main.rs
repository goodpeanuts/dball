use dball_combora::generator::Generator;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    log::info!("Running in terminal mode with concurrent batch generation (10 threads).");
    let generator = Generator::create_generator(Generator::BlueMorn);
    let tickets = generator.generate_batch()?;
    for ticket in tickets {
        log::info!("Ticket: {ticket}");
    }

    Ok(())
}
