pub async fn setup() -> anyhow::Result<nebulon::client::Client> {
  let client = nebulon::client::connect(nebulon::config::ClientConfig {
    auth: Default::default(),
    connection: nebulon::config::ConnectionConfig::Memory,
  })
  .await?;

  client.migrate().await?;

  Ok(client)
}
