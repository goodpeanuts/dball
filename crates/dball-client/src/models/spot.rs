use ansi_term::Colour::{Blue, Green, Red, Yellow};
use chrono::NaiveDateTime;
use dball_combora::dball::{DBall, DBallError, Reward};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Spot record structure for generated ticket numbers
/// The id field will be None for new records and Some(value) for existing records
#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::models::schema::spot)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Spot {
    pub id: Option<i32>,
    pub period: String,
    pub red1: i32,
    pub red2: i32,
    pub red3: i32,
    pub red4: i32,
    pub red5: i32,
    pub red6: i32,
    pub blue: i32,
    pub magnification: i32,
    pub prize_status: Option<i32>,
    pub deprecated: bool,
    pub created_time: NaiveDateTime,
    pub modified_time: NaiveDateTime,
}

impl Spot {
    /// Create a new spot from `DBall` for insertion (id will be None)
    pub fn from_dball(
        period: &str,
        dball: &DBall,
        prize_status: Option<i32>,
    ) -> Result<Self, SpotError> {
        if period.trim().is_empty() {
            return Err(SpotError::EmptyPeriod);
        }

        let now = chrono::Utc::now().naive_utc();

        Ok(Self {
            id: None,
            period: period.to_owned(),
            red1: dball.rball[0] as i32,
            red2: dball.rball[1] as i32,
            red3: dball.rball[2] as i32,
            red4: dball.rball[3] as i32,
            red5: dball.rball[4] as i32,
            red6: dball.rball[5] as i32,
            blue: dball.bball as i32,
            magnification: dball.magnification as i32,
            prize_status,
            deprecated: false,
            created_time: now,
            modified_time: now,
        })
    }

    /// Create a new spot from `DBall` with datetime (for internal use)
    pub fn from_dball_with_datetime(
        period: String,
        dball: &DBall,
        prize_status: Option<i32>,
        created_time: NaiveDateTime,
        modified_time: NaiveDateTime,
    ) -> Result<Self, SpotError> {
        if period.trim().is_empty() {
            return Err(SpotError::EmptyPeriod);
        }

        Ok(Self {
            id: None,
            period,
            red1: dball.rball[0] as i32,
            red2: dball.rball[1] as i32,
            red3: dball.rball[2] as i32,
            red4: dball.rball[3] as i32,
            red5: dball.rball[4] as i32,
            red6: dball.rball[5] as i32,
            blue: dball.bball as i32,
            magnification: dball.magnification as i32,
            prize_status,
            deprecated: false,
            created_time,
            modified_time,
        })
    }

    /// Convert to `DBall` for validation and operations
    pub fn to_dball(&self) -> Result<DBall, SpotError> {
        let red_numbers = self.red_numbers();
        let red_u8: Vec<u8> = red_numbers.iter().map(|&x| x as u8).collect();
        let blue_u8 = self.blue as u8;
        let magnification_usize = self.magnification as usize;

        DBall::new(red_u8, blue_u8, magnification_usize).map_err(SpotError::from)
    }

    /// Validate spot using `DBall`'s validation logic
    pub fn check(&self) -> Result<(), SpotError> {
        // Use DBall for number validation
        self.to_dball()?;

        // Additional spot-specific validations
        if self.period.trim().is_empty() {
            return Err(SpotError::EmptyPeriod);
        }

        Ok(())
    }

    /// Check if spot is valid (returns boolean)
    pub fn is_valid(&self) -> bool {
        self.check().is_ok()
    }

    /// Get formatted created time string
    pub fn formatted_created_time(&self) -> String {
        self.created_time.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Get formatted modified time string
    pub fn formatted_modified_time(&self) -> String {
        self.modified_time.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn red_numbers(&self) -> Vec<i32> {
        vec![
            self.red1, self.red2, self.red3, self.red4, self.red5, self.red6,
        ]
    }

    pub fn blue_number(&self) -> i32 {
        self.blue
    }

    /// Get reward enum based on prize status
    pub fn reward_level(&self) -> anyhow::Result<Option<Reward>> {
        if let Some(num) = self.prize_status {
            let reward = Reward::try_from(num)?;
            Ok(Some(reward))
        } else {
            Ok(None)
        }
    }
}

/// Spot validation error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpotError {
    EmptyPeriod,
    DBallError(String), // Wrapper for DBallError
}

// Convert DBallError to SpotError
impl From<DBallError> for SpotError {
    fn from(err: DBallError) -> Self {
        Self::DBallError(err.to_string())
    }
}

// From DBall to Spot (convenience method)
impl From<DBall> for Spot {
    fn from(dball: DBall) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id: None,
            period: String::new(), // Will need to be set separately
            red1: dball.rball[0] as i32,
            red2: dball.rball[1] as i32,
            red3: dball.rball[2] as i32,
            red4: dball.rball[3] as i32,
            red5: dball.rball[4] as i32,
            red6: dball.rball[5] as i32,
            blue: dball.bball as i32,
            magnification: dball.magnification as i32,
            prize_status: None,
            deprecated: false,
            created_time: now,
            modified_time: now,
        }
    }
}

// TryFrom Spot to DBall
impl TryFrom<Spot> for DBall {
    type Error = SpotError;

    fn try_from(spot: Spot) -> Result<Self, Self::Error> {
        spot.to_dball()
    }
}

// TryFrom &Spot to DBall
impl TryFrom<&Spot> for DBall {
    type Error = SpotError;

    fn try_from(spot: &Spot) -> Result<Self, Self::Error> {
        spot.to_dball()
    }
}

impl PartialEq for Spot {
    fn eq(&self, other: &Self) -> bool {
        self.period == other.period
            && self.red1 == other.red1
            && self.red2 == other.red2
            && self.red3 == other.red3
            && self.red4 == other.red4
            && self.red5 == other.red5
            && self.red6 == other.red6
            && self.blue == other.blue
            && self.magnification == other.magnification
    }
}

impl Display for SpotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPeriod => write!(f, "Period cannot be empty"),
            Self::DBallError(msg) => write!(f, "invalid spot record: {msg}"),
        }
    }
}

impl std::error::Error for SpotError {}

impl std::fmt::Display for Spot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Try to use DBall's display format for the numbers part
        match self.to_dball() {
            Ok(dball) => {
                write!(f, "{dball}")
            }
            Err(_) => {
                // Fallback to manual formatting if DBall conversion fails
                write!(
                    f,
                    "{} {} {} {} {}",
                    Red.bold().paint("invalid numbers spot Record:"),
                    Green.bold().paint(format!("Period:{}", self.period)),
                    Red.bold().paint(format!(
                        "{:02} {:02} {:02} {:02} {:02} {:02}",
                        self.red1, self.red2, self.red3, self.red4, self.red5, self.red6
                    )),
                    Blue.bold().paint(format!("{:02}", self.blue)),
                    Yellow.bold().paint(format!("{}x", self.magnification)),
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_spot_from_dball() -> anyhow::Result<()> {
        let dball = DBall::new(vec![2, 6, 7, 13, 16, 28], 11, 1)
            .map_err(|e| anyhow::anyhow!("DBall creation failed: {}", e))?;
        let test_spot = Spot::from_dball("2025084", &dball, None)?;

        log::info!("created spot success: {test_spot}");
        assert_eq!(test_spot.period, "2025084");
        assert_eq!(test_spot.red_numbers(), vec![2, 6, 7, 13, 16, 28]);
        assert_eq!(test_spot.blue_number(), 11);
        assert_eq!(test_spot.magnification, 1);
        assert_eq!(test_spot.prize_status, None);

        Ok(())
    }

    #[test]
    fn create_spot_from_dball_with_prize() -> anyhow::Result<()> {
        let dball = DBall::new(vec![2, 6, 7, 13, 16, 28], 11, 1)
            .map_err(|e| anyhow::anyhow!("DBall creation failed: {}", e))?;
        let test_spot = Spot::from_dball("2025084", &dball, Some(1))?;

        assert_eq!(test_spot.prize_status, Some(1));
        // Test basic data access only
        assert_eq!(test_spot.red_numbers(), vec![2, 6, 7, 13, 16, 28]);
        assert_eq!(test_spot.blue_number(), 11);

        Ok(())
    }

    #[test]
    fn create_spot_with_empty_period() -> anyhow::Result<()> {
        let dball = DBall::new(vec![2, 6, 7, 13, 16, 28], 11, 1)
            .map_err(|e| anyhow::anyhow!("DBall creation failed: {}", e))?;
        let result = Spot::from_dball("", &dball, None);
        assert!(matches!(result, Err(SpotError::EmptyPeriod)));
        Ok(())
    }

    #[test]
    fn test_spot_to_dball_conversion() -> anyhow::Result<()> {
        let original_dball = DBall::new(vec![2, 6, 7, 13, 16, 28], 11, 2)
            .map_err(|e| anyhow::anyhow!("DBall creation failed: {}", e))?;
        let test_spot = Spot::from_dball("2025084", &original_dball, None)?;

        let converted_dball = test_spot.to_dball()?;
        assert_eq!(converted_dball.rball, [2, 6, 7, 13, 16, 28]);
        assert_eq!(converted_dball.bball, 11);
        assert_eq!(converted_dball.magnification, 2);

        Ok(())
    }

    #[test]
    fn test_try_from_conversions() -> anyhow::Result<()> {
        let original_dball = DBall::new(vec![2, 6, 7, 13, 16, 28], 11, 1)
            .map_err(|e| anyhow::anyhow!("DBall creation failed: {}", e))?;
        let test_spot = Spot::from_dball("2025084", &original_dball, None)?;

        // Test TryFrom Spot to DBall
        let dball_from_spot: DBall = test_spot.clone().try_into()?;
        assert_eq!(dball_from_spot.rball, [2, 6, 7, 13, 16, 28]);

        // Test TryFrom &Spot to DBall
        let dball_from_ref: DBall = (&test_spot).try_into()?;
        assert_eq!(dball_from_ref.rball, [2, 6, 7, 13, 16, 28]);

        Ok(())
    }

    #[test]
    fn test_dball_error_conversion() {
        let dball_error = DBallError::RBallOutOfRange(34);
        let spot_error: SpotError = dball_error.into();
        assert!(matches!(spot_error, SpotError::DBallError(_)));

        let dball_error = DBallError::InvalidBBall(17);
        let spot_error: SpotError = dball_error.into();
        assert!(matches!(spot_error, SpotError::DBallError(_)));
    }

    #[test]
    fn test_spot_basic_methods() -> anyhow::Result<()> {
        let dball = DBall::new(vec![2, 6, 7, 13, 16, 28], 11, 1)
            .map_err(|e| anyhow::anyhow!("DBall creation failed: {}", e))?;
        let test_spot = Spot::from_dball("2025084", &dball, None)?;

        // Test basic getter methods
        assert_eq!(test_spot.red_numbers(), vec![2, 6, 7, 13, 16, 28]);
        assert_eq!(test_spot.blue_number(), 11);

        // Test time formatting methods
        assert!(!test_spot.formatted_created_time().is_empty());
        assert!(!test_spot.formatted_modified_time().is_empty());

        Ok(())
    }
}
