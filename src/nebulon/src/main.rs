#![deny(
  unsafe_code,
  // reason = "Let's just not do it"
)]
#![deny(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::unreachable,
  clippy::arithmetic_side_effects
  // reason = "We have to handle errors properly"
)]
#![deny(
  clippy::dbg_macro,
  // reason = "Use tracing instead"
)]

use nebulon;
use tracing_subscriber::{
  layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[tokio::main]
#[tracing::instrument]
async fn main() -> anyhow::Result<()> {
  let format_layer = tracing_subscriber::fmt::layer();
  let (filter_layer, filter_handle) =
    tracing_subscriber::reload::Layer::new(build_tracing_filter("info")?);
  tracing_subscriber::registry()
    .with(filter_layer)
    .with(format_layer)
    .try_init()?;

  // TODO: from config when loaded if needed
  let log_level = "info".to_string();
  filter_handle.modify(move |filter| {
    #[allow(clippy::unwrap_used)] // NOTE: static and env doesn't change
    let new_filter = build_tracing_filter(log_level.as_str()).unwrap();
    *filter = new_filter;
  })?;

  let client = nebulon::connect(false).await?;
  client.migrate().await?;

  let chat = client.insert_chat().await?;
  let message = client
    .insert_message(chat.id, "Me".to_string(), "Hello, world!".to_string())
    .await?;
  let id = message.id;
  let content = message.content;
  println!("Created message {id} with content {content}");

  let search = client.search_messages("hello").await?;
  for message in search {
    let id = message.record.id;
    let chat = message.record.chat;
    let highlights = message.highlights;
    let score = message.score;
    println!("Found message {id} from chat {chat} with highlights {highlights} and score {score}");
  }

  Ok(())
}

fn build_tracing_filter(level: &str) -> anyhow::Result<EnvFilter> {
  Ok(
    tracing_subscriber::EnvFilter::builder()
      .with_default_directive(tracing::level_filters::LevelFilter::WARN.into())
      .with_env_var("NEBULON_LOG")
      .from_env()?
      .add_directive(format!("nebulon={level}").parse()?),
  )
}
