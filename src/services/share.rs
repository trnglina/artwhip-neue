use chrono::Local;
use tracing::debug;

pub async fn create_share(
  pool: &sqlx::SqlitePool,
  enrollment_id: i64,
) -> Result<(), anyhow::Error> {
  let now = Local::now();
  debug!("Logging share for enrollment {} at {}", enrollment_id, now);

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
