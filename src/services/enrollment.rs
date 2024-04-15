use chrono::{DateTime, Duration, Local};
use poise::serenity_prelude as serenity;
use tracing::info;

pub async fn get_enrollment_id(
  pool: &sqlx::SqlitePool,
  guild_id: &serenity::GuildId,
  user_id: &serenity::UserId,
) -> Result<Option<i64>, anyhow::Error> {
  let guild_id_str = guild_id.get().to_string();
  let user_id_str = user_id.get().to_string();

  let enrollment = sqlx::query!(
    r#"
      SELECT
        MAX(created_at) AS created_at
      , id
      FROM enrollments
      WHERE guild_id = ? AND user_id = ?
    "#,
    guild_id_str,
    user_id_str,
  )
  .fetch_optional(pool)
  .await?;

  Ok(enrollment.map(|e| e.id).flatten())
}

pub async fn create_enrollment(
  pool: &sqlx::SqlitePool,
  chatgpt: &chatgpt::prelude::ChatGPT,
  guild_id: &serenity::GuildId,
  user_id: &serenity::UserId,
  channel_id: &serenity::ChannelId,
  start: Option<&str>,
  interval: Option<i64>,
) -> Result<(DateTime<Local>, i64), anyhow::Error> {
  let now = Local::now();
  let interval_hours: i64 = interval.unwrap_or(24);
  let starting_at = match start {
    Some(start) => {
      let mut conversation = chatgpt.new_conversation_directed(format!(
        r#"You are a date parsing service. You will receive an input in natural
           language, and you must convert it into an RFC 3339 format string,
           taking into account the current time if and only if the input is
           relative. Only resolve times in the future. If the input is a
           relative time in the past, you may output {} as a fallback.
           
           Do not output anything other than the RFC 3339 string.
           
           The current time zone is: +0900 (UTC +0900).
           The current time in RFC 3339 format is: {}."#,
        now + Duration::hours(interval_hours),
        now.to_rfc3339(),
      ));

      let response = conversation.send_message(start).await?;
      DateTime::parse_from_rfc3339(&response.message().content)?.with_timezone(&Local)
    }
    None => now + Duration::hours(interval_hours),
  };

  info!(
    "Enrollment: {}/{} from {}@{} hours",
    user_id, guild_id, starting_at, interval_hours,
  );

  let guild_id_str = guild_id.get().to_string();
  let user_id_str = user_id.get().to_string();
  let channel_id_str = channel_id.get().to_string();

  sqlx::query!(
    r#"
      INSERT INTO
        enrollments (
          guild_id
        , user_id
        , channel_id
        , created_at
        , starting_at
        , interval_hours
      )
      VALUES (?, ?, ?, ?, ?, ?)
    "#,
    guild_id_str,
    user_id_str,
    channel_id_str,
    now,
    starting_at,
    interval_hours,
  )
  .execute(pool)
  .await?;

  Ok((starting_at, interval_hours))
}
