use chrono::{DateTime, Local};

#[derive(Debug)]
pub struct Period {
  pub start: DateTime<Local>,
  pub end: DateTime<Local>,
  pub deadline: DateTime<Local>,
}

pub type PopulatedPeriod = (Period, bool);
