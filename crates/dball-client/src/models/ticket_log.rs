use dball_combora::dball::DBall;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = super::schema::ticket_log)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TicketLog {
    pub code: String,
    pub kj_date: Option<chrono::NaiveDate>,
    pub xq: Option<String>,
    pub number1: Option<i32>,
    pub number2: Option<i32>,
    pub number3: Option<i32>,
    pub number4: Option<i32>,
    pub number5: Option<i32>,
    pub number6: Option<i32>,
    pub number7: Option<i32>,
    pub jsondata: Option<String>,
    pub total_sales: Option<i32>,
    pub jackpot: Option<i32>,
    pub prize1_num: Option<i32>,
    pub prize1_money: Option<i32>,
    pub prize2_num: Option<i32>,
    pub prize2_money: Option<i32>,
    pub prize3_num: Option<i32>,
    pub prize3_money: Option<i32>,
    pub prize4_num: Option<i32>,
    pub prize4_money: Option<i32>,
    pub prize5_num: Option<i32>,
    pub prize5_money: Option<i32>,
    pub prize6_num: Option<i32>,
    pub prize6_money: Option<i32>,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = super::schema::ticket_log)]
pub struct NewTicketLog {
    pub code: String,
    pub kj_date: Option<chrono::NaiveDate>,
    pub xq: Option<String>,
    pub number1: Option<i32>,
    pub number2: Option<i32>,
    pub number3: Option<i32>,
    pub number4: Option<i32>,
    pub number5: Option<i32>,
    pub number6: Option<i32>,
    pub number7: Option<i32>,
    pub jsondata: Option<String>,
    pub total_sales: Option<i32>,
    pub jackpot: Option<i32>,
    pub prize1_num: Option<i32>,
    pub prize1_money: Option<i32>,
    pub prize2_num: Option<i32>,
    pub prize2_money: Option<i32>,
    pub prize3_num: Option<i32>,
    pub prize3_money: Option<i32>,
    pub prize4_num: Option<i32>,
    pub prize4_money: Option<i32>,
    pub prize5_num: Option<i32>,
    pub prize5_money: Option<i32>,
    pub prize6_num: Option<i32>,
    pub prize6_money: Option<i32>,
}

impl TicketLog {
    pub fn red_numbers(&self) -> Vec<i32> {
        [
            self.number1,
            self.number2,
            self.number3,
            self.number4,
            self.number5,
            self.number6,
        ]
        .iter()
        .filter_map(|&x| x)
        .collect()
    }

    pub fn blue_number(&self) -> Option<i32> {
        self.number7
    }

    pub fn all_numbers(&self) -> Vec<i32> {
        let mut numbers = self.red_numbers();
        if let Some(blue) = self.blue_number() {
            numbers.push(blue);
        }
        numbers
    }

    pub fn parse_json_numbers(&self) -> Result<Vec<i32>, serde_json::Error> {
        match &self.jsondata {
            Some(json_str) => serde_json::from_str(json_str),
            None => Ok(vec![]),
        }
    }

    pub fn to_dball(&self) -> anyhow::Result<DBall> {
        let red_numbers = self.red_numbers();
        let red_u8: Vec<u8> = red_numbers.iter().map(|&x| x as u8).collect();
        let blue = self.blue_number().unwrap_or(0) as u8;

        DBall::new_one(red_u8, blue)
            .map_err(|e| anyhow::anyhow!("Failed to convert ticket log to DBall: {e}"))
    }
}

impl std::fmt::Display for TicketLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use console::style;

        let red_numbers = self.red_numbers();
        let blue_number = self.blue_number().unwrap_or(0);

        write!(
            f,
            "period: {}, date: {}, numbers: {} + {}",
            self.code,
            self.kj_date.map_or("unknown".to_owned(), |d| d.to_string()),
            style(format!("{red_numbers:?}")).red().bold(),
            style(blue_number.to_string()).blue().bold()
        )
    }
}

impl TryFrom<TicketLog> for DBall {
    type Error = anyhow::Error;

    fn try_from(ticket_log: TicketLog) -> Result<Self, Self::Error> {
        ticket_log.to_dball()
    }
}
