#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::todo)]
#![deny(clippy::unreachable)]
#![deny(clippy::allow_attributes_without_reason)]

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let config = gravity::config::new_async::<nebulon::config::Config>(
    "NEBULON",
    env!("QUALIFIER"),
    env!("ORGANIZATION"),
    "nebulon",
    concat!(env!("CARGO_PKG_REPOSITORY"), "/src/nebulon"),
  )
  .await;

  let client =
    nebulon::client::connect(config.values_async().await.client.clone())
      .await?;
  client.migrate().await?;

  Ok(())
}
