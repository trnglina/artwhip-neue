use chrono::{DateTime, Local};
use poise::serenity_prelude as serenity;
use sqlx::{Connection, SqliteConnection};
use tracing::{error, info};

use crate::{
  models::enrollment::{Enrollment, PartialEnrollment},
  services::{
    period::{calculate_populated_periods, construct_periods},
    share::get_enrollment_shares,
  },
};

pub struct Reminder {
  pub user_id: serenity::UserId,
  pub channel_id: serenity::ChannelId,
}

pub async fn get_reminders(
  conn: &mut SqliteConnection,
  start: DateTime<Local>,
  end: DateTime<Local>,
) -> Result<Vec<Reminder>, anyhow::Error> {
  let mut tx = conn.begin().await?;

  let enrollments = sqlx::query_as!(
    PartialEnrollment,
    r#"
      SELECT
        id,
        guild_id,
        user_id,
        channel_id,
        created_at as "created_at: DateTime<Local>",
        starting_at as "starting_at: DateTime<Local>",
        interval_hours
      FROM current_enrollments
    "#,
  )
  .fetch_all(&mut *tx)
  .await?;
  info!("Found {} active enrollments", enrollments.len());

  let mut reminders = Vec::new();
  for enrollment in enrollments {
    let enrollment: Enrollment = enrollment.into();
    let periods = construct_periods(&enrollment, end);
    if let Some(last_period) = periods.last() {
      if last_period.end < start {
        continue;
      }

      let shares = get_enrollment_shares(&mut *tx, enrollment.id).await?;
      info!(?shares);
      let populated_periods = calculate_populated_periods(periods, shares);
      info!(?populated_periods);

      if let Some((_, populated)) = populated_periods.last() {
        if !populated {
          let user_id = enrollment
            .user_id
            .parse::<u64>()
            .map(serenity::UserId::from);

          let channel_id = enrollment
            .channel_id
            .parse::<u64>()
            .map(serenity::ChannelId::from);

          if let (Ok(user_id), Ok(channel_id)) = (user_id, channel_id) {
            reminders.push(Reminder {
              user_id,
              channel_id,
            });
          }
        } else {
          info!("Enrollment {} already fulfilled", enrollment.id)
        }
      } else {
        error!(
          "No populated periods found for enrollment {}",
          enrollment.id
        );
      }
    }
  }

  Ok(reminders)
}
