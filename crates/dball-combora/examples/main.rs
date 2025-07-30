use dball_combora::generator::Generator;

fn main() -> anyhow::Result<()> {
    println!("Running in terminal mode. This is a placeholder for terminal-specific logic.");
    use dball_combora::dball::DBallBatch;

    let bluemorn = Generator::create_generator(Generator::BlueMorn);
    let tickets = bluemorn.generate_batch()?;
    let sims = DBallBatch(tickets.to_vec()).cosine_similarity();
    println!("Cosine similarities: {:?}", sims);
    println!("Generated tickets:\n{}", DBallBatch(tickets.to_vec()));

    Ok(())
}
