use crate::db::{spot, tickets};
use crate::models::Spot;
use crate::service::ticket::update_this_year_ticket;
use chrono::{DateTime, Datelike as _, Duration, TimeZone as _, Utc, Weekday};
use dball_combora::dball::DBall;
use std::collections::HashMap;

use super::ticket;

pub async fn next_draw_time(time: Option<DateTime<Utc>>) -> anyhow::Result<DateTime<Utc>> {
    const BEIJING_OFFSET_HOURS: i64 = 8;
    let open_time = chrono::NaiveTime::from_hms_opt(21, 20, 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid draw time"))?;

    let base_time = time.unwrap_or_else(Utc::now);
    let beijing_time = base_time + Duration::hours(BEIJING_OFFSET_HOURS);
    let current_weekday = beijing_time.weekday();

    let days_offset = match current_weekday {
        Weekday::Mon | Weekday::Wed | Weekday::Sat => 1,
        Weekday::Fri => 2,
        Weekday::Thu => {
            if beijing_time.time() < open_time {
                0
            } else {
                3
            }
        }
        Weekday::Tue | Weekday::Sun => {
            if beijing_time.time() < open_time {
                0
            } else {
                2
            }
        }
    };

    let next_draw_beijing = beijing_time.date_naive() + Duration::days(days_offset);
    let next_draw_time = next_draw_beijing.and_time(open_time);

    let next_draw_utc =
        Utc.from_utc_datetime(&next_draw_time) - Duration::hours(BEIJING_OFFSET_HOURS);

    Ok(next_draw_utc)
}

pub async fn update_all_unprize_spots() -> anyhow::Result<Vec<Spot>> {
    let spots = spot::get_all_unprize_spots()?;

    if spots.is_empty() {
        log::info!("No unprized spots found, nothing to update");
        return get_prized_spots().await;
    }

    // Update the current year's ticket to ensure we have the latest data
    update_this_year_ticket().await?;

    let next_period = ticket::get_next_period().await?;

    log::debug!("Found {} unprized spots", spots.len());
    let mut spots_by_period: HashMap<String, Vec<(i32, DBall, Option<i32>)>> = HashMap::new();
    for spot in spots {
        if spot.period == next_period {
            log::debug!("Skipping spot for next period",);
            continue; // Skip spots for the upcoming period
        }
        spots_by_period
            .entry(spot.period.clone())
            .or_default()
            .push((
                spot.id.expect(crate::NEVER_NONE_BY_DATABASE),
                TryFrom::try_from(spot.clone())?,
                spot.prize_status, // Include current prize status
            ));
    }

    let mut errors = Vec::new();
    #[expect(clippy::iter_over_hash_type)]
    for (spot_period, dballs_to_check) in spots_by_period {
        log::debug!(
            "Processing {} spots for period {spot_period}",
            dballs_to_check.len()
        );

        let opened_ball = if let Some(t) = tickets::get_ticket_by_period(&spot_period)? {
            t.to_dball()?
        } else {
            log::warn!("No ticket found for period {spot_period}, Failed to update unprized spots");
            continue;
        };

        // update the spot by checking with the opened dball
        for dball_to_check in dballs_to_check {
            let reward_price = dball_to_check.1.check_prize(&opened_ball).to_i32();

            match spot::update_spot_prize_status_by_id(dball_to_check.0, Some(reward_price)) {
                Ok(()) => {
                    log::debug!(
                        "Updated spot for id {id} with reward level {reward_price}",
                        id = dball_to_check.0
                    );
                }
                Err(e) => {
                    errors.push(e.to_string());
                }
            }
        }
    }

    if !errors.is_empty() {
        let e = errors.join("\n");
        anyhow::bail!("Failed to update some spots:\n{e}");
    }

    log::info!("Completed updating all spots");
    get_prized_spots().await
}

pub async fn generate_batch_spots() -> anyhow::Result<()> {
    use dball_combora::generator::RandomGenerator as _;

    let generator = dball_combora::generator::bluemorn::BlueMorn;
    if get_next_period_unprized_spots().await?.len().ge(&10) {
        log::warn!("There are already more than 10 unprized spots, skipping generation");
        return Ok(());
    }

    let tickets = generator.generate_batch()?;
    insert_new_spots_batch_to_next_period(&tickets).await?;
    Ok(())
}

pub async fn insert_new_spots_batch_to_next_period(dballs: &[DBall]) -> anyhow::Result<()> {
    let next_period = ticket::get_next_period().await?;

    for dball in dballs {
        spot::insert_spot_from_dball(&next_period, dball, None)?;
    }
    Ok(())
}

pub async fn deprecated_last_batch_unprized_spot() -> anyhow::Result<usize> {
    use crate::db::spot;

    // Get the latest 5 unprized spots (prize_status = None)
    let latest_unprized_spots = spot::get_latest_unprized_spots(5)?;

    if latest_unprized_spots.is_empty() {
        log::info!("No unprized spots found to deprecate");
        return Ok(0);
    }

    // Extract IDs, only considering spots with id
    let spot_ids: Vec<i32> = latest_unprized_spots
        .into_iter()
        .filter_map(|s| s.id)
        .collect();

    if spot_ids.is_empty() {
        log::warn!("No valid spot IDs found for deprecation");
        return Ok(0);
    }

    log::info!(
        "Marking {} spots as deprecated: {:?}",
        spot_ids.len(),
        spot_ids
    );

    let updated_count = spot::mark_spots_deprecated(&spot_ids)?;

    log::info!("Successfully marked {updated_count} spots as deprecated");
    Ok(updated_count)
}

pub async fn get_prized_spots() -> anyhow::Result<Vec<Spot>> {
    use crate::db::spot;
    let prized_spots = spot::get_all_spots()?
        .into_iter()
        .filter_map(|s| match s.prize_status {
            Some(_) => Some(s), // All spots with prize status including deprecated
            _ => None,
        })
        .collect::<Vec<Spot>>();

    Ok(prized_spots)
}

/// Excluding deprecated spots
pub async fn get_next_period_unprized_spots() -> anyhow::Result<Vec<Spot>> {
    use crate::db::spot;
    let next_period = ticket::get_next_period().await?;

    let unprized_spots = spot::get_spots_by_period(&next_period)?
        .iter()
        .filter_map(|s| {
            if s.prize_status.is_none() && !s.deprecated {
                Some(s.clone())
            } else {
                None
            }
        })
        .collect::<Vec<Spot>>();

    Ok(unprized_spots)
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDate;

    #[tokio::test]
    async fn bluemorn_insert_dball_batch() -> anyhow::Result<()> {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let generator = dball_combora::generator::bluemorn::BlueMorn;
        let tickets = generator.generate_multiple(5);
        insert_new_spots_batch_to_next_period(&tickets).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_next_draw_time() -> anyhow::Result<()> {
        // 测试当前时间为None的情况
        let next_time = next_draw_time(None).await?;
        log::debug!("Next draw time from now: {}", next_time);

        // 测试周一下午 (应该返回周二晚上21:20)
        let monday = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(); // 2024-01-01是周一
        let monday_afternoon = monday.and_hms_opt(15, 0, 0).unwrap();
        let monday_utc = Utc.from_utc_datetime(&monday_afternoon);

        let next_time_from_monday = next_draw_time(Some(monday_utc)).await?;
        log::debug!(
            "Next draw time from Monday afternoon: {}",
            next_time_from_monday
        );

        // 验证结果应该是周二北京时间21:20 (UTC时间13:20)
        let expected_tuesday = monday.succ_opt().unwrap().and_hms_opt(13, 20, 0).unwrap();
        let expected_tuesday_utc = Utc.from_utc_datetime(&expected_tuesday);
        assert_eq!(next_time_from_monday, expected_tuesday_utc);

        // 测试周二早上 (应该返回当天晚上21:20)
        let tuesday_morning = monday.succ_opt().unwrap().and_hms_opt(8, 0, 0).unwrap();
        let tuesday_morning_utc = Utc.from_utc_datetime(&tuesday_morning);

        let next_time_from_tuesday_morning = next_draw_time(Some(tuesday_morning_utc)).await?;
        log::debug!(
            "Next draw time from Tuesday morning: {}",
            next_time_from_tuesday_morning
        );

        // 应该是同一天的21:20北京时间
        assert_eq!(next_time_from_tuesday_morning, expected_tuesday_utc);

        // 测试周二后 (应该返回周四21:20)
        let tuesday_night = monday.succ_opt().unwrap().and_hms_opt(22, 0, 0).unwrap();
        let tuesday_night_utc = Utc.from_utc_datetime(&tuesday_night);

        let next_time_from_tuesday_night = next_draw_time(Some(tuesday_night_utc)).await?;
        log::debug!(
            "Next draw time from Tuesday night: {}",
            next_time_from_tuesday_night
        );

        // 应该是周四北京时间21:20 (UTC时间13:20)
        let expected_thursday = monday
            .succ_opt()
            .unwrap()
            .succ_opt()
            .unwrap()
            .succ_opt()
            .unwrap()
            .and_hms_opt(13, 20, 0)
            .unwrap();
        let expected_thursday_utc = Utc.from_utc_datetime(&expected_thursday);
        assert_eq!(next_time_from_tuesday_night, expected_thursday_utc);

        // 测试周五 (应该返回周日21:20)
        let friday = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(); // 2024-01-05是周五
        let friday_afternoon = friday.and_hms_opt(15, 0, 0).unwrap();
        let friday_utc = Utc.from_utc_datetime(&friday_afternoon);

        let next_time_from_friday = next_draw_time(Some(friday_utc)).await?;
        log::debug!("Next draw time from Friday: {}", next_time_from_friday);

        // 应该是周日北京时间21:20 (UTC时间13:20)
        let expected_sunday = friday
            .succ_opt()
            .unwrap()
            .succ_opt()
            .unwrap()
            .and_hms_opt(13, 20, 0)
            .unwrap();
        let expected_sunday_utc = Utc.from_utc_datetime(&expected_sunday);
        assert_eq!(next_time_from_friday, expected_sunday_utc);

        Ok(())
    }
}
