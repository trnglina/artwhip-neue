use chrono::{DateTime, Duration, Local};
use poise::serenity_prelude as serenity;
use tracing::debug;

pub struct Reminder {
  pub user_id: serenity::UserId,
  pub channel_id: serenity::ChannelId,
}

pub async fn get_reminders(
  pool: &sqlx::SqlitePool,
  start: DateTime<Local>,
  end: DateTime<Local>,
) -> Result<Vec<Reminder>, anyhow::Error> {
  let mut tx = pool.begin().await?;

  let enrollments = sqlx::query!(
    r#"
      SELECT
        MAX(created_at) AS created_at
      , id
      , guild_id
      , user_id
      , channel_id
      , starting_at as "starting_at: DateTime<Local>"
      , interval_hours
      FROM enrollments
      WHERE starting_at < ?
      GROUP BY guild_id, user_id
    "#,
    end
  )
  .fetch_all(&mut *tx)
  .await?;

  debug!("Found {} relevant enrollments", enrollments.len());

  let mut reminders = Vec::new();
  for enrollment in enrollments {
    let starting_at = enrollment.starting_at.unwrap();
    let interval_hours = enrollment.interval_hours.unwrap();

    let period_n = (end - starting_at).num_hours() / interval_hours;
    debug!(
      "Enrollment {} at period {}",
      enrollment.id.unwrap(),
      period_n
    );
    let effective_end = starting_at + Duration::hours(interval_hours * period_n);

    if effective_end < start || effective_end >= end {
      debug!(
        "Enrollment outside range: {} < {} <= {}",
        start, effective_end, end
      );
      continue;
    }

    if let Some(_) = sqlx::query!(
      r#"
        SELECT id
        FROM shares
        WHERE enrollment_id = ? AND created_at <= ?
      "#,
      enrollment.id,
      effective_end
    )
    .fetch_optional(&mut *tx)
    .await?
    {
      debug!("Enrollment already shared at {}", effective_end);
      continue;
    }

    let user_id = enrollment
      .user_id
      .unwrap()
      .parse::<u64>()
      .map(serenity::UserId::from);

    let channel_id = enrollment
      .channel_id
      .unwrap()
      .parse::<u64>()
      .map(serenity::ChannelId::from);

    if let (Ok(user_id), Ok(channel_id)) = (user_id, channel_id) {
      reminders.push(Reminder {
        user_id,
        channel_id,
      });
    }
  }

  Ok(reminders)
}
