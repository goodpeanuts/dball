fn main() {
    use dball_combora::dball::DBall;

    let mut cnt: u128 = 0;
    loop {
        cnt += 1;
        let target_ticket = DBall::generate_random();
        let ticket = DBall::generate_random();
        if target_ticket == ticket {
            println!("Found a matching ticket: {ticket}");
            println!("Total attempts: {cnt}");
            break;
        }
    }
}
