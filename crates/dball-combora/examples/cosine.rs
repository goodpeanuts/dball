fn main() -> anyhow::Result<()> {
    println!("Running in terminal mode. This is a placeholder for terminal-specific logic.");
    use dball_combora::dball::DBallBatch;
    use dball_combora::generator::RandomGenerator as _;

    let tickets = dball_combora::dball::DBall::generate_batch()?;
    let sims = DBallBatch(tickets.to_vec()).cosine_similarity();
    println!("Cosine similarities: {:?}", sims);
    println!("Generated tickets:\n{}", DBallBatch(tickets.to_vec()));

    Ok(())
}
