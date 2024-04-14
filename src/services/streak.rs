use chrono::{DateTime, Duration, Local};
use tracing::debug;

pub async fn get_streak(pool: &sqlx::SqlitePool, enrollment_id: i64) -> Result<i64, anyhow::Error> {
  let enrollment = sqlx::query!(
    r#"
      SELECT
        id
      , created_at as "created_at: DateTime<Local>"
      , starting_at as "starting_at: DateTime<Local>"
      , interval_hours
      FROM enrollments
      WHERE id = ?
    "#,
    enrollment_id,
  )
  .fetch_one(pool)
  .await?;

  let shares = sqlx::query!(
    r#"
      SELECT created_at as "created_at: DateTime<Local>"
      FROM shares
      WHERE enrollment_id = ?
      ORDER BY created_at
    "#,
    enrollment_id,
  )
  .fetch_all(pool)
  .await?;
  debug!(
    "Found {} shares for enrollment {}",
    shares.len(),
    enrollment_id
  );

  let mut streak = 0;
  let mut last_end = enrollment.created_at;
  for share in shares {
    let end = enrollment.starting_at
      + Duration::hours(enrollment.interval_hours * streak)
      // + 50% of interval as leeway
      + Duration::hours(enrollment.interval_hours / 2);
    if share.created_at > last_end && share.created_at <= end {
      streak += 1;
    }

    last_end = end;
  }

  Ok(streak)
}
