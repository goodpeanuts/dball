use ansi_term::Colour::{Blue, Red};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

const COST_PER_TICKET: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Wrapper type for displaying a vector of `DBall`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DBallBatch(pub Vec<DBall>);

impl DBallBatch {
    pub fn to_batch(self) -> anyhow::Result<[DBall; 5]> {
        self.0.try_into().map_err(|e| {
            anyhow::anyhow!("Failed to convert Vec<DBall> to [DBall; 5]:\n{}", Self(e))
        })
    }

    pub fn cosine_similarity(&self) -> Vec<f32> {
        fn calc_cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
            let dot = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
            let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();

            if norm_a == 0.0 || norm_b == 0.0 {
                0.0
            } else {
                dot / (norm_a * norm_b)
            }
        }

        let vectors: Vec<_> = self.0.iter().map(DBall::to_vector).collect();
        let len = vectors.len();
        let mut sims = Vec::new();

        for i in 0..len {
            for j in i + 1..len {
                sims.push(calc_cosine_similarity(&vectors[i], &vectors[j]));
            }
        }
        sims
    }
}

impl Display for DBallBatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|ball| ball.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DBallError {
    InvalidRBallCount(usize),
    InvalidBBall(u8),
    InvalidRBallRange((u8, u8)),
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
            Self::InvalidRBallRange((min, max)) => {
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

    /// Convert a `DBall` to a vector representation for cosine calculations
    /// Red balls are represented as indices 0-32 (1-33)
    /// Blue ball is represented as index 33-48 (1-16)
    pub fn to_vector(ball: &Self) -> Vec<f32> {
        let mut vec = vec![0.0f32; 49];
        for &num in &ball.rball {
            vec[(num - 1) as usize] = 1.0;
        }
        vec[(ball.bball - 1 + 33) as usize] = 1.0;
        vec
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

impl TryFrom<i32> for Reward {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            300_000.. => Ok(Self::FirstPrize),
            140_000..300_000 => Ok(Self::SecondPrize),
            3_000 => Ok(Self::ThirdPrize),
            200 => Ok(Self::FourthPrize),
            100 => Ok(Self::FifthPrize),
            5 => Ok(Self::SixthPrize),
            0 => Ok(Self::NoWin),
            _ => Err(anyhow::anyhow!(
                "Invalid prize value converted to Reward: {value}"
            )),
        }
    }
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

    /// Convert reward to i32 value for database storage
    pub fn to_i32(&self) -> i32 {
        self.prize_amount() as i32
    }
}
