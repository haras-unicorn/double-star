#[derive(clap::Args)]
pub struct FromArgs {}

impl gravity::config::FromArgs for FromArgs {}

#[derive(Default, serde::Deserialize)]
pub struct FromEnv {
  pub db: nebulon::config::ClientConfig,
}

impl gravity::config::FromEnv for FromEnv {}

#[derive(
  Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct FromFile {
  /// Orbitus UI config
  pub ui: orbitus::config::UiConfig,
}

impl gravity::config::FromFile for FromFile {}

#[derive(Clone)]
pub struct Config {
  pub db: nebulon::config::ClientConfig,
  pub ui: orbitus::config::UiConfig,
}

impl gravity::config::Values for Config {
  type TArgs = FromArgs;
  type TEnv = FromEnv;
  type TFile = FromFile;

  fn new(_: Self::TArgs, env: Self::TEnv) -> Self {
    Self {
      db: env.db,
      ui: Default::default(),
    }
  }

  fn import(&mut self, file: Self::TFile) {
    self.ui = file.ui;
  }

  fn export(&self) -> Self::TFile {
    Self::TFile {
      ui: self.ui.clone(),
    }
  }
}
