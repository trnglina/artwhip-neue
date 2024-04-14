use crate::{
  services::{
    enrollment::{create_enrollment, get_enrollment_id},
    streak::get_streak,
  },
  Context,
};

/// Enroll in the art challenge!
#[poise::command(slash_command)]
pub async fn enroll(
  ctx: Context<'_>,
  #[description = "At what time should the challenge start? (default: now)"] //
  start: Option<String>,
  #[description = "Once every how many hours do you want to share? (default: 24)"] //
  interval: Option<i64>,
) -> Result<(), anyhow::Error> {
  if let Some(guild_id) = ctx.guild_id() {
    let (start, interval) = create_enrollment(
      &ctx.data().pool,
      &ctx.data().chatgpt,
      &guild_id,
      &ctx.author().id,
      &ctx.channel_id(),
      start.as_deref(),
      interval,
    )
    .await?;

    ctx
      .reply(format!(
        "Great! Your first deadline will be {}, and then every {} hours after that. Good luck!",
        start.format("on %e %b, %Y at %H:%M"),
        interval,
      ))
      .await?;
  }

  Ok(())
}

/// Check your streak!
#[poise::command(slash_command)]
pub async fn streak(ctx: Context<'_>) -> Result<(), anyhow::Error> {
  if let Some(guild_id) = ctx.guild_id() {
    if let Some(enrollment_id) =
      get_enrollment_id(&ctx.data().pool, &guild_id, &ctx.author().id).await?
    {
      let streak = get_streak(&ctx.data().pool, enrollment_id).await?;

      if streak == 0 {
        ctx
          .reply("You haven't shared anything yet! Come back when you've made some art.")
          .await?;
      } else if streak <= 3 {
        ctx
          .reply(format!(
            "You're on a {} day streak. That's not very impressive, but it might be, if you keep it up!", 
            streak
          ))
          .await?;
      } else {
        ctx
          .reply(format!(
            "You're on a {} day streak. Keep up the good work!",
            streak
          ))
          .await?;
      }
    } else {
      ctx
        .reply("Hey, you're not even enrolled! Join the challenge with `/enroll` you coward.")
        .await?;
    }
  }

  Ok(())
}
