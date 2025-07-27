use serde::Deserialize;

pub const DEFAULT_LOTTERY_CODE: &str = "ssq";

#[derive(Debug, Deserialize, Clone)]
pub struct LotteryData {
    #[serde(rename = "openCode")]
    pub open_code: String,
    pub code: String,
    #[serde(rename = "expect")]
    pub period: String,
    pub name: String,
    pub time: String,
}

impl TryFrom<LotteryData> for crate::models::Ticket {
    type Error = anyhow::Error;

    fn try_from(data: LotteryData) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = data.open_code.split('+').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid open_code format: {}",
                data.open_code
            ));
        }

        let red_balls: Result<Vec<i32>, _> = parts[0]
            .split(',')
            .map(|s| s.trim().parse::<i32>())
            .collect();

        let red_balls =
            red_balls.map_err(|e| anyhow::anyhow!("Failed to parse red balls: {}", e))?;

        let blue_ball: i32 = parts[1]
            .trim()
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse blue ball: {}", e))?;

        Ok(Self::new(data.period, &data.time, &red_balls, blue_ball)?)
    }
}

impl TryFrom<&LotteryData> for crate::models::Ticket {
    type Error = anyhow::Error;

    fn try_from(data: &LotteryData) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = data.open_code.split('+').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid open_code format: {}",
                data.open_code
            ));
        }

        let red_balls: Result<Vec<i32>, _> = parts[0]
            .split(',')
            .map(|s| s.trim().parse::<i32>())
            .collect();

        let red_balls =
            red_balls.map_err(|e| anyhow::anyhow!("Failed to parse red balls: {}", e))?;

        let blue_ball: i32 = parts[1]
            .trim()
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse blue ball: {}", e))?;

        Ok(Self::new(
            data.period.clone(),
            &data.time,
            &red_balls,
            blue_ball,
        )?)
    }
}
