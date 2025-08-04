fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    use dball_combora::dball::DBallBatch;
    use dball_combora::generator::Generator;

    let bluemorn = Generator::create_generator(Generator::BlueMorn);
    let tickets = bluemorn.generate_batch()?;
    let sims = DBallBatch(tickets.to_vec()).cosine_similarity();
    println!("Cosine similarities: {sims:?}");
    println!("Generated tickets:\n{}", DBallBatch(tickets.to_vec()));

    Ok(())
}
