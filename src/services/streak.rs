use chrono::{DateTime, Local};
use sqlx::SqliteConnection;
use tracing::info;

use crate::{
  models::enrollment::Enrollment,
  services::{
    period::{calculate_populated_periods, construct_periods},
    share::get_enrollment_shares,
  },
};

pub async fn get_streak(
  conn: &mut SqliteConnection,
  enrollment: &Enrollment,
  until: DateTime<Local>,
) -> Result<usize, anyhow::Error> {
  info!(
    "Calculating streak for enrollment {} until {}",
    enrollment.id, until
  );

  let periods = construct_periods(&enrollment, until);
  let shares = get_enrollment_shares(conn, enrollment.id).await?;
  info!(?shares);
  let populated_periods = calculate_populated_periods(periods, shares);
  info!(?populated_periods);

  let streak = populated_periods
    .into_iter()
    .rev()
    .take_while(|(_, fulfilled)| *fulfilled)
    .count();

  Ok(streak)
}
