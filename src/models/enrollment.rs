use chrono::{DateTime, Local};
use partially::Partial;

#[derive(Partial, Clone, Debug)]
#[partially(derive(Debug))]
pub struct Enrollment {
  pub id: i64,
  pub guild_id: String,
  pub user_id: String,
  pub channel_id: String,
  pub created_at: DateTime<Local>,
  pub starting_at: DateTime<Local>,
  pub interval_hours: i64,
}

impl From<PartialEnrollment> for Enrollment {
  fn from(partial: PartialEnrollment) -> Self {
    Self {
      id: partial.id.unwrap(),
      guild_id: partial.guild_id.unwrap(),
      user_id: partial.user_id.unwrap(),
      channel_id: partial.channel_id.unwrap(),
      created_at: partial.created_at.unwrap(),
      starting_at: partial.starting_at.unwrap(),
      interval_hours: partial.interval_hours.unwrap(),
    }
  }
}
