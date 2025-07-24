use ansi_term::Colour::{Blue, Red};
use std::fmt::Display;

const COST_PER_TICKET: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DBall {
    pub rball: [u8; 6],
    pub bball: u8,
    pub magnification: usize,
}

impl Display for DBall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            Red.bold().paint(format!(
                "{} {} {} {} {} {}",
                self.rball[0],
                self.rball[1],
                self.rball[2],
                self.rball[3],
                self.rball[4],
                self.rball[5]
            )),
            Blue.bold().paint(format!("{}", self.bball))
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DBallError {
    InvalidRBallCount(usize),
    InvalidBBall(u8),
    InvaildRBallRange((u8, u8)),
    RBallOutOfRange(u8),
    RBallDuplicate,
}

impl Display for DBallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRBallCount(count) => {
                write!(f, "Invalid number of red balls: expected 6, got {count}")
            }
            Self::RBallOutOfRange(ball) => {
                write!(f, "Red ball {ball} is out of range (1-33)")
            }
            Self::RBallDuplicate => write!(f, "Duplicate red balls found"),
            Self::InvalidBBall(ball) => {
                write!(f, "Blue ball {ball} is out of range (1-16)")
            }
            Self::InvaildRBallRange((min, max)) => {
                write!(
                    f,
                    "Red ball range must at least contain 6 numbers, but got: {min}-{max}",
                )
            }
        }
    }
}

impl DBall {
    pub fn new(
        rball: impl AsMut<[u8]>,
        bball: u8,
        magnification: usize,
    ) -> anyhow::Result<Self, DBallError> {
        Self::check(rball, bball, magnification)
    }
    pub fn new_one(rball: impl AsMut<[u8]>, bball: u8) -> anyhow::Result<Self, DBallError> {
        Self::new(rball, bball, 1)
    }

    fn check(
        mut rball: impl AsMut<[u8]>,
        bball: u8,
        magnification: usize,
    ) -> anyhow::Result<Self, DBallError> {
        let rball = rball.as_mut();
        if rball.len() != 6 {
            return Err(DBallError::InvalidRBallCount(rball.len()));
        }

        for ball in rball.iter_mut() {
            if *ball < 1 || *ball > 33 {
                return Err(DBallError::RBallOutOfRange(*ball));
            }
        }

        if !(1..=16).contains(&bball) {
            return Err(DBallError::InvalidBBall(bball));
        }

        rball.sort_unstable();
        let rball: [u8; 6] = if rball.windows(2).any(|w| w[0] == w[1]) {
            return Err(DBallError::RBallDuplicate);
        } else {
            rball
                .try_into()
                .map_err(|_e| DBallError::InvalidRBallCount(rball.len()))?
        };

        Ok(Self {
            rball,
            bball,
            magnification,
        })
    }

    pub fn cost(&self) -> usize {
        self.magnification * COST_PER_TICKET
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reward {
    FirstPrize,
    SecondPrize,
    ThirdPrize,
    FourthPrize,
    FifthPrize,
    SixthPrize,
    NoWin,
}

impl Reward {
    /// get the prize amount
    pub fn prize_amount(&self) -> u32 {
        match self {
            Self::FirstPrize => 4_500_000,
            Self::SecondPrize => 150_000,
            Self::ThirdPrize => 3_000,
            Self::FourthPrize => 200,
            Self::FifthPrize => 10,
            Self::SixthPrize => 5,
            Self::NoWin => 0,
        }
    }

    /// get the prize description
    pub fn description(&self) -> &'static str {
        match self {
            Self::FirstPrize => "#1",
            Self::SecondPrize => "#2",
            Self::ThirdPrize => "#3",
            Self::FourthPrize => "#4",
            Self::FifthPrize => "#5",
            Self::SixthPrize => "#6",
            Self::NoWin => "#0",
        }
    }
}
