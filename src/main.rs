use std::vec;

use chatgpt::{
  config::{ChatGPTEngine, ModelConfigurationBuilder},
  prelude::ChatGPT,
};
use poise::{
  serenity_prelude::{self as serenity},
  FrameworkError,
};
use services::{enrollment::get_enrollment, share::create_share};
use sqlx::{migrate::Migrator, SqlitePool};
use tracing::error;

mod commands;
mod models;
mod services;

static MIGRATOR: Migrator = sqlx::migrate!();

#[derive(Clone, Debug)]
pub struct Data {
  pub pool: SqlitePool,
  pub chatgpt: ChatGPT,
}

pub type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

/*
fn spawn_reminders_task(ctx: serenity::Context, data: Data) {
  task::spawn(async move {
    let mut previous_time: DateTime<Local> = Local::now();
    let mut interval = time::interval(Duration::from_secs(30));
    loop {
      interval.tick().await;

      let now = Local::now();
      if now <= previous_time {
        continue;
      }

      let mut conn = data.pool.acquire().await.unwrap();
      match get_reminders(&mut conn, previous_time, now).await {
        Ok(reminders) => {
          for reminder in reminders {
            let gif = get_gif("judgemental cat", 10).await.ok().flatten();
            let embeds = gif.map(|url| json!([{ "image": { "url": url } }]));

            match ctx
              .http()
              .send_message(
                reminder.channel_id,
                Vec::new(),
                &json!({
                  "content": format!("<@{}>, don't forget to post an update", reminder.user_id),
                  "embeds": embeds,
                }),
              )
              .await
            {
              Ok(_) => (),
              Err(err) => error!("Failed to send reminder: {}", err),
            }
          }
        }
        Err(err) => error!("Failed to get reminders: {}", err),
      }

      previous_time = now;
    }
  });
}
*/

async fn setup(
  ctx: &serenity::Context,
  _ready: &serenity::Ready,
  framework: &poise::Framework<Data, anyhow::Error>,
) -> Result<Data, anyhow::Error> {
  poise::builtins::register_globally(ctx, &framework.options().commands).await?;

  let database_url = std::env::var("DATABASE_URL")?;
  let pool = SqlitePool::connect(&database_url).await?;

  let openai_api_key = std::env::var("OPENAI_API_KEY")?;
  let chatgpt = ChatGPT::new_with_config(
    &openai_api_key,
    ModelConfigurationBuilder::default()
      .engine(ChatGPTEngine::Gpt35Turbo)
      .build()?,
  )?;

  MIGRATOR.run(&pool.clone()).await?;
  let data = Data { pool, chatgpt };

  // spawn_reminders_task(ctx.clone(), data.clone());
  Ok(data)
}

async fn handle_error(error: FrameworkError<'_, Data, anyhow::Error>) {
  error!(?error);

  if let Some(ctx) = error.ctx() {
    if let Err(err) = ctx
      .reply(format!("An error was encountered: {}", error))
      .await
    {
      error!("Failed to send error message: {}", err)
    }
  }
}

async fn handle_event(
  ctx: &serenity::Context,
  event: &serenity::FullEvent,
  data: &Data,
) -> Result<(), anyhow::Error> {
  match event {
    serenity::FullEvent::Message { new_message } => {
      if new_message.author.bot || !new_message.content.contains("+log") {
        return Ok(());
      }

      let mut conn = data.pool.acquire().await.unwrap();
      if let Some(guild_id) = new_message.guild_id {
        if let Ok(Some(enrollment)) =
          get_enrollment(&mut conn, &guild_id, &new_message.author.id).await
        {
          create_share(&data.pool, enrollment.id).await?;
          new_message.react(ctx, '🎨').await?;
        }
      }

      Ok(())
    }
    _ => Ok(()),
  }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<(), anyhow::Error> {
  tracing_subscriber::fmt::init();
  let _ = dotenvy::dotenv();

  let token = std::env::var("DISCORD_TOKEN")?;
  let intents = serenity::GatewayIntents::non_privileged()
    | serenity::GatewayIntents::MESSAGE_CONTENT
    | serenity::GatewayIntents::GUILD_MEMBERS;

  let framework = poise::Framework::<Data, anyhow::Error>::builder()
    .options(poise::FrameworkOptions {
      on_error: |error| Box::pin(async move { handle_error(error).await }),
      event_handler: |ctx, event, _, data| Box::pin(async { handle_event(ctx, event, data).await }),
      commands: vec![commands::enroll(), commands::streak()],
      ..Default::default()
    })
    .setup(|ctx, ready, framework| Box::pin(async move { setup(ctx, ready, framework).await }))
    .build();

  serenity::ClientBuilder::new(token, intents)
    .framework(framework)
    .await?
    .start()
    .await?;

  Ok(())
}
