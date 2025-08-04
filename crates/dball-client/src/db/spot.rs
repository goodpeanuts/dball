use crate::db::get_db_connection;
use crate::models::Spot;
use crate::models::schema::spot;
use dball_combora::dball::DBall;
use diesel::prelude::*;

/// Insert a new spot from `DBall`
pub fn insert_spot_from_dball(
    period: &str,
    dball: &DBall,
    prize_status: Option<i32>,
) -> anyhow::Result<()> {
    let new_spot = Spot::from_dball(period, dball, prize_status)
        .map_err(|e| anyhow::anyhow!("Error creating spot from DBall: {e}"))?;
    insert_spot(&new_spot)
}

pub fn insert_spot(new_spot: &Spot) -> anyhow::Result<()> {
    let mut connection = get_db_connection()?;
    diesel::insert_into(spot::table)
        .values(new_spot)
        .execute(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error inserting spot: {e}"))
        .and_then(|count| {
            if count != 1 {
                Err(anyhow::anyhow!(
                    "Expected to insert exactly one spot, but inserted {}",
                    count
                ))
            } else {
                Ok(())
            }
        })
}

/// Should update only one spot's prize status
pub fn update_spot_prize_status_by_id(id: i32, prize_status: Option<i32>) -> anyhow::Result<()> {
    let mut connection = get_db_connection()?;
    diesel::update(spot::table.filter(spot::id.eq(id)))
        .set((
            spot::prize_status.eq(prize_status),
            spot::modified_time.eq(chrono::Utc::now().naive_utc()),
        ))
        .execute(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error updating spot prize status: {e}"))
        .and_then(|count| {
            if count != 1 {
                Err(anyhow::anyhow!(
                    "Expected to update exactly one spot, but updated {count}",
                ))
            } else {
                Ok(())
            }
        })
}

/// Mark spots as deprecated (deprecated = true)
/// Only marks spots that are currently not deprecated
pub fn mark_spots_deprecated(spot_ids: &[i32]) -> anyhow::Result<usize> {
    if spot_ids.is_empty() {
        return Ok(0);
    }

    let mut connection = get_db_connection()?;

    // Update only spots that are currently not deprecated
    let updated_count = diesel::update(
        spot::table
            .filter(spot::id.eq_any(spot_ids))
            .filter(spot::deprecated.eq(false)),
    )
    .set((
        spot::deprecated.eq(true),
        spot::modified_time.eq(chrono::Utc::now().naive_utc()),
    ))
    .execute(&mut connection)
    .map_err(|e| anyhow::anyhow!("Error marking spots as deprecated: {e}"))?;

    log::debug!(
        "Marked {} spots as deprecated out of {} requested",
        updated_count,
        spot_ids.len()
    );
    Ok(updated_count)
}

/// Get spots by period and convert them to `DBall`
pub fn get_spots_by_period_as_dball(period: &str) -> anyhow::Result<Vec<DBall>> {
    let spots = get_spots_by_period(period)?;
    let mut result = Vec::new();

    for spot in spots {
        match spot.to_dball() {
            Ok(dball) => result.push(dball),
            Err(e) => log::warn!(
                "Failed to convert spot {} to DBall: {e}",
                spot.id.unwrap_or(-1)
            ),
        }
    }

    Ok(result)
}

pub fn get_all_spots() -> anyhow::Result<Vec<Spot>> {
    let mut connection = get_db_connection()?;
    spot::table
        .load::<Spot>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error loading spots: {e}"))
}

pub fn get_all_unprize_spots() -> anyhow::Result<Vec<Spot>> {
    let mut connection = get_db_connection()?;
    spot::table
        .filter(spot::prize_status.is_null())
        .load::<Spot>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error loading spots: {e}"))
}

pub fn get_spots_by_period(period: &str) -> anyhow::Result<Vec<Spot>> {
    let mut connection = get_db_connection()?;
    spot::table
        .filter(spot::period.eq(period))
        .load::<Spot>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error finding spots for period {period}: {e}"))
}

pub fn get_latest_spots(limit: i64) -> anyhow::Result<Vec<Spot>> {
    let mut connection = get_db_connection()?;
    spot::table
        .order(spot::created_time.desc())
        .limit(limit)
        .load::<Spot>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error loading latest {limit} spots: {e}"))
}

pub fn get_latest_unprized_spots(limit: i64) -> anyhow::Result<Vec<Spot>> {
    let mut connection = get_db_connection()?;
    spot::table
        .filter(spot::prize_status.is_null())
        .order(spot::created_time.desc())
        .limit(limit)
        .load::<Spot>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error loading latest {limit} unprized spots: {e}"))
}

pub fn find_spots_with_red_number(number: i32) -> anyhow::Result<Vec<Spot>> {
    let mut connection = get_db_connection()?;
    spot::table
        .filter(
            spot::red1
                .eq(number)
                .or(spot::red2.eq(number))
                .or(spot::red3.eq(number))
                .or(spot::red4.eq(number))
                .or(spot::red5.eq(number))
                .or(spot::red6.eq(number)),
        )
        .order(spot::id.desc())
        .load::<Spot>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error finding spots with red number {number}: {e}"))
}

pub fn find_spots_with_blue_number(blue: i32) -> anyhow::Result<Vec<Spot>> {
    let mut connection = get_db_connection()?;
    spot::table
        .filter(spot::blue.eq(blue))
        .order(spot::id.desc())
        .load::<Spot>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error finding spots with blue number {blue}: {e}"))
}

pub fn find_spots_by_prize_status(status: Option<i32>) -> anyhow::Result<Vec<Spot>> {
    let mut connection = get_db_connection()?;
    spot::table
        .filter(spot::prize_status.eq(status))
        .order(spot::id.desc())
        .load::<Spot>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error finding spots with prize status {status:?}: {e}"))
}

pub fn find_winning_spots() -> anyhow::Result<Vec<Spot>> {
    let mut connection = get_db_connection()?;
    spot::table
        .filter(spot::prize_status.is_not_null())
        .filter(spot::prize_status.gt(0))
        .order(spot::prize_status.asc())
        .load::<Spot>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error finding winning spots: {e}"))
}

pub fn find_spots_by_magnification(magnification: i32) -> anyhow::Result<Vec<Spot>> {
    let mut connection = get_db_connection()?;
    spot::table
        .filter(spot::magnification.eq(magnification))
        .order(spot::id.desc())
        .load::<Spot>(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error finding spots with magnification {magnification}: {e}"))
}

pub fn count_spots() -> anyhow::Result<i64> {
    let mut connection = get_db_connection()?;
    spot::table
        .count()
        .get_result(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error counting spots: {e}"))
}

pub fn count_spots_by_period(period: &str) -> anyhow::Result<i64> {
    let mut connection = get_db_connection()?;
    spot::table
        .filter(spot::period.eq(period))
        .count()
        .get_result(&mut connection)
        .map_err(|e| anyhow::anyhow!("Error counting spots for period {period}: {e}"))
}

#[cfg(test)]
mod test {
    use super::*;
    use dball_combora::dball::DBall;

    #[test]
    fn all_spots() -> anyhow::Result<()> {
        // Retrieve all spots
        match get_all_spots() {
            Ok(spots) => {
                log::info!("Successfully retrieved {count} spots:", count = spots.len());
                for spot in &spots {
                    log::info!("{spot}");
                }

                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    #[test]
    fn test_count_spots() -> anyhow::Result<()> {
        match count_spots() {
            Ok(count) => {
                log::info!("Total spots count: {count}");
                assert!(count >= 0);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to count spots: {e}");
                Err(e)
            }
        }
    }

    #[test]
    fn test_get_latest_spots() -> anyhow::Result<()> {
        match get_latest_spots(5) {
            Ok(spots) => {
                log::info!("Latest 5 spots:");
                for spot in &spots {
                    log::info!("{spot}");
                }
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to get latest spots: {e}");
                Err(e)
            }
        }
    }

    #[test]
    fn test_find_spots_by_period() -> anyhow::Result<()> {
        let period = "2025084";
        match get_spots_by_period(period) {
            Ok(spots) => {
                log::info!("Found {} spots for period {}", spots.len(), period);
                for spot in &spots {
                    log::info!("{spot}");
                }
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to find spots by period {period}: {e}");
                Err(e)
            }
        }
    }

    #[test]
    fn test_find_spots_with_red_number() -> anyhow::Result<()> {
        let red_number = 13;
        match find_spots_with_red_number(red_number) {
            Ok(spots) => {
                log::info!(
                    "Found {} spots containing red number {}",
                    spots.len(),
                    red_number
                );
                for spot in &spots {
                    log::info!("{spot}");
                }
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to find spots with red number {red_number}: {e}");
                Err(e)
            }
        }
    }

    #[test]
    fn test_find_spots_with_blue_number() -> anyhow::Result<()> {
        let blue_number = 11;
        match find_spots_with_blue_number(blue_number) {
            Ok(spots) => {
                log::info!(
                    "Found {} spots with blue number {}",
                    spots.len(),
                    blue_number
                );
                for spot in &spots {
                    log::info!("{spot}");
                }
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to find spots with blue number {blue_number}: {e}");
                Err(e)
            }
        }
    }

    #[test]
    fn test_find_spots_by_prize_status() -> anyhow::Result<()> {
        // Test finding spots waiting for results (None)
        match find_spots_by_prize_status(None) {
            Ok(spots) => {
                log::debug!("Found {} spots waiting for results", spots.len());
                spots.iter().take(3).for_each(|spot| {
                    log::debug!("{spot}");
                });
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to find spots by prize status: {e}");
                Err(e)
            }
        }
    }

    #[test]
    fn test_find_winning_spots() -> anyhow::Result<()> {
        match find_winning_spots() {
            Ok(spots) => {
                log::info!("Found {count} winning spots", count = spots.len());
                for spot in &spots {
                    log::info!("{spot}");
                }
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to find winning spots: {e}");
                Err(e)
            }
        }
    }

    #[test]
    fn test_count_spots_by_period() -> anyhow::Result<()> {
        let period = "2025084";
        match count_spots_by_period(period) {
            Ok(count) => {
                log::info!("Found {count} spots for period {period}");
                assert!(count >= 0);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to count spots by period {period}: {e}");
                Err(e)
            }
        }
    }

    #[test]
    fn test_insert_spot_from_dball() -> anyhow::Result<()> {
        let dball = DBall::new(vec![5, 10, 15, 20, 25, 30], 8, 1)
            .map_err(|e| anyhow::anyhow!("DBall creation failed: {e}"))?;
        let period = "2025085".to_owned();

        match insert_spot_from_dball(&period, &dball, None) {
            Ok(()) => {
                log::info!("Successfully inserted spot from DBall");

                // Verify the spot was inserted by checking count
                let count = count_spots_by_period(&period)?;
                assert!(count >= 1);

                Ok(())
            }
            Err(e) => {
                log::error!("Failed to insert spot from DBall: {e}");
                Err(e)
            }
        }
    }

    #[test]
    fn test_mark_spots_deprecated() -> anyhow::Result<()> {
        // First insert some test spots
        let dball1 = DBall::new(vec![1, 2, 3, 4, 5, 6], 1, 1)
            .map_err(|e| anyhow::anyhow!("DBall creation failed: {e}"))?;
        let dball2 = DBall::new(vec![7, 8, 9, 10, 11, 12], 2, 1)
            .map_err(|e| anyhow::anyhow!("DBall creation failed: {e}"))?;

        let period = "2025999";
        insert_spot_from_dball(period, &dball1, None)?;
        insert_spot_from_dball(period, &dball2, None)?;

        // Get the inserted spots
        let spots = get_spots_by_period(period)?;
        assert!(spots.len() >= 2);

        // Extract some IDs (take first 2)
        let spot_ids: Vec<i32> = spots.iter().take(2).filter_map(|s| s.id).collect();
        assert!(!spot_ids.is_empty());

        // Mark them as deprecated
        let updated_count = mark_spots_deprecated(&spot_ids)?;
        log::info!("Marked {updated_count} spots as deprecated");

        // Verify they were marked
        let updated_spots = get_spots_by_period(period)?;
        let deprecated_count = updated_spots.iter().filter(|s| s.deprecated).count();

        assert!(deprecated_count > 0);
        log::info!("Found {deprecated_count} deprecated spots after marking");

        Ok(())
    }

    #[test]
    fn test_get_latest_unprized_spots() -> anyhow::Result<()> {
        let spots = get_latest_unprized_spots(3)?;
        log::info!("Found {} latest unprized spots", spots.len());

        for spot in &spots {
            assert!(
                spot.prize_status.is_none(),
                "Spot should have no prize status"
            );
            log::info!("Unprized spot: {} - {:?}", spot.period, spot.id);
        }

        Ok(())
    }
}
