use chrono::{DateTime, Duration, Local};
use tracing::info;

use crate::models::{
  enrollment::Enrollment,
  period::{Period, PopulatedPeriod},
  share::Share,
};

pub fn construct_periods(enrollment: &Enrollment, until: DateTime<Local>) -> Vec<Period> {
  let mut end = enrollment.starting_at;
  let mut periods = Vec::new();
  while end <= until {
    periods.push(Period {
      start: end - Duration::hours(enrollment.interval_hours),
      end,
      deadline: end + Duration::seconds(enrollment.interval_hours * 1800),
    });

    end += Duration::hours(enrollment.interval_hours);
  }

  periods
}

pub fn calculate_populated_periods(
  periods: Vec<Period>,
  shares: Vec<Share>,
) -> Vec<PopulatedPeriod> {
  let mut iter = shares.into_iter();
  let mut current_share = iter.next();
  let mut populated = Vec::new();
  for period in periods {
    let mut fulfilled = false;
    while let Some(share) = current_share {
      info!(?share);
      current_share = iter.next();
      if share.created_at > period.start && share.created_at <= period.deadline {
        fulfilled = true;
        break;
      }
    }

    populated.push((period, fulfilled))
  }

  populated
}
