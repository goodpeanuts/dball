use diesel::{BoolExpressionMethods as _, ExpressionMethods as _, QueryDsl as _, RunQueryDsl as _};

use crate::{
    db::get_db_connection,
    models::{TicketLog, schema::ticket_log},
};

pub fn get_all_records() -> anyhow::Result<Vec<TicketLog>> {
    let mut connection = get_db_connection()?;
    ticket_log::table
        .load::<TicketLog>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error loading records: {}", e))
}

pub fn get_record_by_code(record_code: &str) -> anyhow::Result<Option<TicketLog>> {
    let mut connection = get_db_connection()?;
    ticket_log::table
        .filter(ticket_log::code.eq(record_code))
        .load::<TicketLog>(&mut connection)
        .and_then(|results| match results.len() {
            0 => Ok(None),
            1 => Ok(results.first().cloned()),
            _ => Err(diesel::result::Error::QueryBuilderError(Box::new(
                std::io::Error::other("Multiple records found, but expected only one"),
            ))),
        })
        .map_err(|e| anyhow::anyhow!("Error finding record with code {}: {}", record_code, e))
}

pub fn get_records_by_date_range(
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
) -> anyhow::Result<Vec<TicketLog>> {
    let mut connection = get_db_connection()?;
    ticket_log::table
        .filter(ticket_log::kj_date.ge(start_date))
        .filter(ticket_log::kj_date.le(end_date))
        .order(ticket_log::kj_date.asc())
        .load::<TicketLog>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error loading records by date range: {}", e))
}

pub fn get_latest_records(limit: i64) -> anyhow::Result<Vec<TicketLog>> {
    let mut connection = get_db_connection()?;
    ticket_log::table
        .order(ticket_log::kj_date.desc())
        .limit(limit)
        .load::<TicketLog>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error loading latest {} records: {}", limit, e))
}

pub fn find_records_with_number(number: i32) -> anyhow::Result<Vec<TicketLog>> {
    let mut connection = get_db_connection()?;
    ticket_log::table
        .filter(
            ticket_log::number1
                .eq(number)
                .or(ticket_log::number2.eq(number))
                .or(ticket_log::number3.eq(number))
                .or(ticket_log::number4.eq(number))
                .or(ticket_log::number5.eq(number))
                .or(ticket_log::number6.eq(number))
                .or(ticket_log::number7.eq(number)),
        )
        .order(ticket_log::kj_date.desc())
        .load::<TicketLog>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error finding records with number {}: {}", number, e))
}

pub fn count_records() -> anyhow::Result<i64> {
    let mut connection = get_db_connection()?;
    ticket_log::table
        .count()
        .get_result(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error counting records: {}", e))
}

pub fn get_max_jackpot_record() -> anyhow::Result<TicketLog> {
    let mut connection = get_db_connection()?;
    ticket_log::table
        .order(ticket_log::jackpot.desc())
        .first::<TicketLog>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error finding max jackpot record: {}", e))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_count_records() {
        match count_records() {
            Ok(count) => log::info!("count: {count}"),
            Err(e) => panic!("count records failed: {e}"),
        };
    }

    #[test]
    fn retrive_latest_five_records() {
        match get_latest_records(5) {
            Ok(records) => {
                records.iter().for_each(|r| log::info!("{r}"));
            }
            Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn find_records_by_period() {
        match get_record_by_code("2003001") {
            Ok(record) => match record {
                Some(record) => match record.parse_json_numbers() {
                    Ok(json_numbers) => log::info!("json numbers: {json_numbers:?}"),
                    Err(e) => log::error!("parse json numbers failed: {e}"),
                },
                None => log::info!("record not found"),
            },
            Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn find_records_contain_number() {
        match find_records_with_number(1) {
            Ok(records) => {
                records.iter().for_each(|r| log::info!("{r}"));
            }
            Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn retrive_max_jackpot_record() {
        match get_max_jackpot_record() {
            Ok(record) => {
                log::info!("{record}");
            }
            Err(e) => panic!("{e}"),
        }
    }
}
