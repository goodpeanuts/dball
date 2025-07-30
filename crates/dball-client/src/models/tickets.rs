use ansi_term::Colour::{Blue, Green, Red};
use chrono::NaiveDateTime;
use dball_combora::dball::DBall;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Complete ticket record structure for both querying and inserting
/// The id field will be None for new records and Some(value) for existing records
#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::models::schema::tickets)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Ticket {
    pub id: Option<i32>,
    pub period: String,
    pub time: NaiveDateTime,
    pub red1: i32,
    pub red2: i32,
    pub red3: i32,
    pub red4: i32,
    pub red5: i32,
    pub red6: i32,
    pub blue: i32,
    pub created_time: NaiveDateTime,
    pub modified_time: NaiveDateTime,
}

impl Ticket {
    /// Create a new ticket for insertion (id will be None)
    pub fn new(
        period: String,
        time_str: &str,
        red_numbers: &[i32],
        blue: i32,
    ) -> Result<Self, TicketError> {
        if red_numbers.len() != 6 {
            return Err(TicketError::InvalidRedBallCount(red_numbers.len()));
        }

        // Parse time string to NaiveDateTime
        let time = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S")
            .map_err(|e| TicketError::InvalidTimeFormat(e.to_string()))?;

        let now = chrono::Utc::now().naive_utc();

        let ticket = Self {
            id: None,
            period,
            time,
            red1: red_numbers[0],
            red2: red_numbers[1],
            red3: red_numbers[2],
            red4: red_numbers[3],
            red5: red_numbers[4],
            red6: red_numbers[5],
            blue,
            created_time: now,
            modified_time: now,
        };

        ticket.check()?;
        Ok(ticket)
    }

    /// Create a new ticket with parsed datetime (for internal use)
    pub fn with_datetime(
        period: String,
        time: NaiveDateTime,
        red_numbers: &[i32],
        blue: i32,
    ) -> Result<Self, TicketError> {
        if red_numbers.len() != 6 {
            return Err(TicketError::InvalidRedBallCount(red_numbers.len()));
        }

        let now = chrono::Utc::now().naive_utc();

        let ticket = Self {
            id: None,
            period,
            time,
            red1: red_numbers[0],
            red2: red_numbers[1],
            red3: red_numbers[2],
            red4: red_numbers[3],
            red5: red_numbers[4],
            red6: red_numbers[5],
            blue,
            created_time: now,
            modified_time: now,
        };

        ticket.check()?;
        Ok(ticket)
    }

    /// Validate ticket numbers
    pub fn check(&self) -> Result<(), TicketError> {
        // Check red ball numbers
        let red_numbers = self.red_numbers();

        // Check red ball range (1-33)
        for &ball in &red_numbers {
            if ball < 1 || ball > 33 {
                return Err(TicketError::RedBallOutOfRange(ball));
            }
        }

        // Check for duplicate red balls
        let mut sorted_red = red_numbers.clone();
        sorted_red.sort_unstable();
        if sorted_red.windows(2).any(|w| w[0] == w[1]) {
            return Err(TicketError::RedBallDuplicate);
        }

        // Check blue ball range (1-16)
        if self.blue < 1 || self.blue > 16 {
            return Err(TicketError::BlueBallOutOfRange(self.blue));
        }

        // Check period is not empty
        if self.period.trim().is_empty() {
            return Err(TicketError::EmptyPeriod);
        }

        // Note: No need to check time format since it's already validated as NaiveDateTime

        Ok(())
    }

    /// Check if ticket is valid (returns boolean)
    pub fn is_valid(&self) -> bool {
        self.check().is_ok()
    }

    /// Get formatted time string
    pub fn formatted_time(&self) -> String {
        self.time.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Get time as `NaiveDateTime`
    pub fn get_time(&self) -> NaiveDateTime {
        self.time
    }

    pub fn red_numbers(&self) -> Vec<i32> {
        vec![
            self.red1, self.red2, self.red3, self.red4, self.red5, self.red6,
        ]
    }

    /// Convert ticket to `DBall` format for prize checking
    pub fn to_dball(&self) -> anyhow::Result<DBall> {
        let red_numbers = self.red_numbers();
        let red_u8: Vec<u8> = red_numbers.iter().map(|&x| x as u8).collect();

        DBall::new_one(red_u8, self.blue as u8)
            .map_err(|e| anyhow::anyhow!("Failed to convert ticket to DBall: {e}"))
    }

    pub fn blue_number(&self) -> i32 {
        self.blue
    }

    pub fn all_numbers(&self) -> Vec<i32> {
        let mut numbers = self.red_numbers();
        numbers.push(self.blue);
        numbers
    }

    pub fn format_numbers(&self) -> String {
        format!(
            "{:02} {:02} {:02} {:02} {:02} {:02} + {:02}",
            self.red1, self.red2, self.red3, self.red4, self.red5, self.red6, self.blue
        )
    }
}

/// Ticket validation error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TicketError {
    InvalidRedBallCount(usize),
    RedBallOutOfRange(i32),
    RedBallDuplicate,
    BlueBallOutOfRange(i32),
    EmptyPeriod,
    InvalidTimeFormat(String),
}

impl PartialEq for Ticket {
    fn eq(&self, other: &Self) -> bool {
        self.period == other.period
            && self.red1 == other.red1
            && self.red2 == other.red2
            && self.red3 == other.red3
            && self.red4 == other.red4
            && self.red5 == other.red5
            && self.red6 == other.red6
            && self.blue == other.blue
    }
}

impl TryFrom<Ticket> for DBall {
    type Error = anyhow::Error;

    fn try_from(ticket: Ticket) -> Result<Self, Self::Error> {
        ticket.to_dball()
    }
}

impl TryFrom<&Ticket> for DBall {
    type Error = anyhow::Error;

    fn try_from(ticket: &Ticket) -> Result<Self, Self::Error> {
        ticket.to_dball()
    }
}

impl Display for TicketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRedBallCount(count) => {
                write!(f, "Invalid number of red balls: expected 6, got {count}")
            }
            Self::RedBallOutOfRange(ball) => {
                write!(f, "Red ball {ball} is out of range (1-33)")
            }
            Self::RedBallDuplicate => write!(f, "Duplicate red balls found"),
            Self::BlueBallOutOfRange(ball) => {
                write!(f, "Blue ball {ball} is out of range (1-16)")
            }
            Self::EmptyPeriod => write!(f, "Period cannot be empty"),
            Self::InvalidTimeFormat(time_str) => write!(f, "Invalid time format: {time_str}"),
        }
    }
}

impl std::error::Error for TicketError {}

impl std::fmt::Display for Ticket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            Green.bold().paint(format!(
                "{} {}",
                self.period,
                self.time.format("%Y-%m-%d %H:%M:%S")
            )),
            Red.bold().paint(format!(
                "{:02} {:02} {:02} {:02} {:02} {:02}",
                self.red1, self.red2, self.red3, self.red4, self.red5, self.red6
            )),
            Blue.bold().paint(format!("{:02}", self.blue))
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn create_ticket_datetime_fields() -> anyhow::Result<()> {
        // Create a test ticket
        let test_ticket = Ticket::new(
            "2018005".to_string(),
            "2018-11-20 21:18:20",
            &[5, 12, 18, 25, 30, 33],
            15,
        )?;

        log::info!("created ticket success: {test_ticket}");

        Ok(())
    }

    #[test]
    fn create_ticket_with_valid_time_format() {
        let time_str = "2018-11-20 21:18:20";
        let red_numbers = [1, 5, 12, 18, 23, 31];

        let t = Ticket::new("2018001".to_string(), time_str, &red_numbers, 8);
        assert!(t.is_ok());
    }

    #[test]
    fn create_ticket_with_invalid_time_format() {
        let red_numbers = [1, 5, 12, 18, 23, 31];

        let t = Ticket::new("2018002".to_string(), "invalid-time", &red_numbers, 8);
        assert!(matches!(t, Err(TicketError::InvalidTimeFormat(_))));
    }
}
