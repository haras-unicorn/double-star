#[derive(clap::Args)]
pub struct FromArgs {}

impl gravity::config::FromArgs for FromArgs {}

#[derive(Default, serde::Deserialize)]
pub struct FromEnv {
  #[serde(flatten)]
  pub client: ClientConfig,
}

impl gravity::config::FromEnv for FromEnv {}

#[derive(
  Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct FromFile {}

impl gravity::config::FromFile for FromFile {}

#[derive(Clone)]
pub struct Config {
  pub client: ClientConfig,
}

impl gravity::config::Values for Config {
  type TArgs = FromArgs;

  type TEnv = FromEnv;

  type TFile = FromFile;

  fn new(_: Self::TArgs, env: Self::TEnv) -> Self {
    Self { client: env.client }
  }

  fn import(&mut self, _: Self::TFile) {}

  fn export(&self) -> Self::TFile {
    Self::TFile {}
  }
}

#[derive(Default, Clone, serde::Deserialize)]
pub struct ClientConfig {
  #[serde(flatten)]
  pub auth: AuthConfig,
  #[serde(flatten)]
  pub connection: ConnectionConfig,
}

#[derive(derivative::Derivative, Clone, serde::Deserialize)]
#[derivative(Default)]
pub struct AuthConfig {
  #[derivative(Default(value = "\"double_star\".to_string()"))]
  pub user: String,
  #[derivative(Default(value = "\"double_star\".to_string()"))]
  pub pass: String,
}

#[derive(derivative::Derivative, Clone, serde::Deserialize)]
#[derivative(Default)]
#[serde(untagged)]
pub enum ConnectionConfig {
  #[derivative(Default)]
  Websocket(WebsocketConnectionConfig),
  Embedded(EmbeddedConnectionConfig),
  Memory,
}

#[derive(derivative::Derivative, Clone, serde::Deserialize)]
#[derivative(Default)]
pub struct WebsocketConnectionConfig {
  #[derivative(Default(value = "\"localhost\".to_string()"))]
  pub host: String,
  #[derivative(Default(value = "8000"))]
  pub port: u32,
}

#[derive(Default, Clone, serde::Deserialize)]
pub struct EmbeddedConnectionConfig {
  pub path: Option<std::path::PathBuf>,
}
