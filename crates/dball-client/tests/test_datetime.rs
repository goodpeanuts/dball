#[test]
fn test() -> anyhow::Result<()> {
    use dball_client::models::Ticket;
    // Test creating a ticket with the new datetime format
    let time_str = "2018-11-20 21:18:20";
    let red_numbers = [1, 5, 12, 18, 23, 31];

    match Ticket::new("2018001".to_string(), time_str, &red_numbers, 8) {
        Ok(ticket) => {
            println!("✅ Successfully created ticket:");
            println!("   Period: {}", ticket.period);
            println!(
                "   Time: {} (parsed from '{}')",
                ticket.formatted_time(),
                time_str
            );
            println!("   Red balls: {:?}", ticket.red_numbers());
            println!("   Blue ball: {}", ticket.blue_number());
            println!("   Display: {}", ticket);
            println!("   Valid: {}", ticket.is_valid());
        }
        Err(e) => {
            println!("❌ Failed to create ticket: {}", e);
        }
    }

    // Test with invalid time format
    println!("\n--- Testing invalid time format ---");
    match Ticket::new("2018002".to_string(), "invalid-time", &red_numbers, 8) {
        Ok(_) => println!("❌ Should have failed with invalid time format"),
        Err(e) => println!("✅ Correctly rejected invalid time: {}", e),
    }

    Ok(())
}

#[test]
#[ignore = "cannote open database connection in test"]
fn test_insert_into_db() -> anyhow::Result<()> {
    use dball_client::models::Ticket;
    use dball_client::service::ticket::{get_all_tickets, insert_ticket};

    println!("=== Testing Database Operations with DateTime ===\n");

    // Create a test ticket
    let test_ticket = Ticket::new(
        "2018003".to_string(),
        "2018-11-20 21:18:20",
        &[3, 7, 15, 22, 28, 33],
        12,
    )?;

    println!("✅ Created test ticket: {}", test_ticket);

    // Insert into database
    println!("\n--- Inserting ticket into database ---");
    match insert_ticket(&test_ticket) {
        Ok(()) => println!("✅ Successfully inserted ticket into database"),
        Err(e) => {
            println!("❌ Failed to insert ticket: {}", e);
            return Err(e);
        }
    }

    // Retrieve all tickets
    println!("\n--- Retrieving all tickets from database ---");
    match get_all_tickets() {
        Ok(tickets) => {
            println!("✅ Successfully retrieved {} tickets:", tickets.len());
            for (i, ticket) in tickets.iter().enumerate() {
                println!("   {}. {}", i + 1, ticket);
                println!("      Datetime: {}", ticket.get_time());
                println!("      Formatted: {}", ticket.formatted_time());
            }
        }
        Err(e) => {
            println!("❌ Failed to retrieve tickets: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
