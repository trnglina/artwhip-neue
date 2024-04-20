use chrono::{DateTime, Local};
use sqlx::SqliteConnection;
use tracing::info;

use crate::models::share::Share;

pub async fn get_enrollment_shares(
  conn: &mut SqliteConnection,
  enrollment_id: i64,
) -> Result<Vec<Share>, anyhow::Error> {
  Ok(
    sqlx::query_as!(
      Share,
      r#"
        SELECT
          id,
          enrollment_id,
          created_at as "created_at: DateTime<Local>"
        FROM shares
        WHERE enrollment_id = ?
        ORDER BY created_at
      "#,
      enrollment_id,
    )
    .fetch_all(conn)
    .await?,
  )
}

pub async fn create_share(
  pool: &sqlx::SqlitePool,
  enrollment_id: i64,
) -> Result<(), anyhow::Error> {
  let now = Local::now();
  info!("Logging share for enrollment {} at {}", enrollment_id, now);

  sqlx::query!(
    r#"
      INSERT INTO shares (enrollment_id, created_at)
      VALUES (?, ?)
    "#,
    enrollment_id,
    now,
  )
  .execute(pool)
  .await?;

  Ok(())
}
