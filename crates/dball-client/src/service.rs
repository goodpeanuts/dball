use crate::datastore::establish_connection;
use crate::datastore::models::TicketLog;
use crate::datastore::schema::ticket_log;
use diesel::prelude::*;

pub fn get_all_records() -> anyhow::Result<Vec<TicketLog>> {
    let mut connection = establish_connection()?;
    ticket_log::table
        .load::<TicketLog>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error loading records: {}", e))
}

/// 根据代码获取单条记录
pub fn get_record_by_code(record_code: &str) -> anyhow::Result<TicketLog> {
    let mut connection = establish_connection()?;
    ticket_log::table
        .filter(ticket_log::code.eq(record_code))
        .first::<TicketLog>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error finding record with code {}: {}", record_code, e))
}

/// 获取指定日期范围内的记录
pub fn get_records_by_date_range(
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
) -> anyhow::Result<Vec<TicketLog>> {
    let mut connection = establish_connection()?;
    ticket_log::table
        .filter(ticket_log::kj_date.ge(start_date))
        .filter(ticket_log::kj_date.le(end_date))
        .order(ticket_log::kj_date.asc())
        .load::<TicketLog>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error loading records by date range: {}", e))
}

/// 获取最新的 N 条记录
pub fn get_latest_records(limit: i64) -> anyhow::Result<Vec<TicketLog>> {
    let mut connection = establish_connection()?;
    ticket_log::table
        .order(ticket_log::kj_date.desc())
        .limit(limit)
        .load::<TicketLog>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error loading latest {} records: {}", limit, e))
}

/// 根据号码查找包含特定号码的记录
pub fn find_records_with_number(number: i32) -> anyhow::Result<Vec<TicketLog>> {
    let mut connection = establish_connection()?;
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

/// 统计记录总数
pub fn count_records() -> anyhow::Result<i64> {
    let mut connection = establish_connection()?;
    ticket_log::table
        .count()
        .get_result(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error counting records: {}", e))
}

/// 获取最大回报金额的记录
pub fn get_max_jackpot_record() -> anyhow::Result<TicketLog> {
    let mut connection = establish_connection()?;
    ticket_log::table
        .order(ticket_log::jackpot.desc())
        .first::<TicketLog>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error finding max jackpot record: {}", e))
}
