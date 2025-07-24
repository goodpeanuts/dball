fn main() {
    use dball_combora::dball::DBall;

    println!("Running in terminal mode. This is a placeholder for terminal-specific logic.");

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut selected_tickets = Vec::new();

    while selected_tickets.len() < 5 {
        let tickets = DBall::generate_multiple(100);

        let should_pick = rng.gen_bool(0.2);

        if should_pick && !tickets.is_empty() {
            let random_index = rng.gen_range(0..tickets.len());
            let selected = tickets[random_index].clone();
            selected_tickets.push(selected);
        }
    }
    selected_tickets.iter().for_each(|ticket| {
        println!("{ticket}");
    });
}
