use chrono::{DateTime, Local};
use partially::Partial;

#[derive(Partial, Clone, Debug)]
#[partially(derive(Debug))]
pub struct Share {
  pub id: i64,
  pub enrollment_id: i64,
  pub created_at: DateTime<Local>,
}
