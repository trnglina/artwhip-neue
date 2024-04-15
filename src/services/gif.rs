use anyhow::anyhow;
use rand::seq::IteratorRandom;
use serde_json::Value;
use tracing::info;

fn select_url(response: Value) -> Result<Option<String>, anyhow::Error> {
  let results = response.get("results").ok_or(anyhow!("No `results`"))?;

  let mut urls = Vec::new();
  for result in results.as_array().ok_or(anyhow!("`results` not array"))? {
    urls.push(
      result
        .get("media_formats")
        .ok_or(anyhow!("No `media_formats`"))?
        .get("gif")
        .ok_or(anyhow!("No `gif`"))?
        .get("url")
        .ok_or(anyhow!("No `url`"))?
        .as_str()
        .ok_or(anyhow!("`url` not string"))?
        .to_owned(),
    );
  }

  Ok(urls.into_iter().choose(&mut rand::thread_rng()))
}

pub async fn get_gif(query: &str, sample: u32) -> Result<Option<String>, anyhow::Error> {
  let url = format!(
    "https://tenor.googleapis.com/v2/search?q={}&key={}&limit={}",
    query,
    std::env::var("TENOR_API_KEY")?,
    sample,
  );
  info!(?url);

  let response = reqwest::get(url).await?.json::<serde_json::Value>().await?;
  info!(?response);

  select_url(response)
}
