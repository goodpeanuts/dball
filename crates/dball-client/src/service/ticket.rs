use crate::db::{establish_connection, get_db_connection};
use crate::models::Ticket;
use crate::models::schema::tickets;
use diesel::prelude::*;

pub fn insert_ticket(new_ticket: &Ticket) -> anyhow::Result<()> {
    let mut connection = get_db_connection()?;
    diesel::insert_into(tickets::table)
        .values(new_ticket)
        .execute(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error inserting ticket: {}", e))
        .and_then(|count| {
            if count != 1 {
                Err(anyhow::anyhow!(
                    "Expected to insert exactly one ticket, but inserted {}",
                    count
                ))
            } else {
                Ok(())
            }
        })
}

pub fn get_all_tickets() -> anyhow::Result<Vec<Ticket>> {
    let mut connection = establish_connection()?;
    tickets::table
        .load::<Ticket>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error loading tickets: {}", e))
}

pub fn get_tickets_by_period(period: &str) -> anyhow::Result<Vec<Ticket>> {
    let mut connection = establish_connection()?;
    tickets::table
        .filter(tickets::period.eq(period))
        .load::<Ticket>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error finding tickets for period {}: {}", period, e))
}

pub fn get_latest_tickets(limit: i64) -> anyhow::Result<Vec<Ticket>> {
    let mut connection = establish_connection()?;
    tickets::table
        .order(tickets::time.desc())
        .limit(limit)
        .load::<Ticket>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error loading latest {} tickets: {}", limit, e))
}

pub fn find_tickets_with_red_number(number: i32) -> anyhow::Result<Vec<Ticket>> {
    let mut connection = establish_connection()?;
    tickets::table
        .filter(
            tickets::red1
                .eq(number)
                .or(tickets::red2.eq(number))
                .or(tickets::red3.eq(number))
                .or(tickets::red4.eq(number))
                .or(tickets::red5.eq(number))
                .or(tickets::red6.eq(number)),
        )
        .order(tickets::id.desc())
        .load::<Ticket>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error finding tickets with red number {}: {}", number, e))
}

pub fn find_tickets_with_blue_number(blue: i32) -> anyhow::Result<Vec<Ticket>> {
    let mut connection = establish_connection()?;
    tickets::table
        .filter(tickets::blue.eq(blue))
        .order(tickets::id.desc())
        .load::<Ticket>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error finding tickets with blue number {}: {}", blue, e))
}

pub fn count_tickets() -> anyhow::Result<i64> {
    let mut connection = establish_connection()?;
    tickets::table
        .count()
        .get_result(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error counting tickets: {}", e))
}
