#[derive(clap::Args)]
pub struct FromArgs {}

impl gravity::config::FromArgs for FromArgs {}

#[derive(Default, serde::Deserialize)]
pub struct FromEnv {
  pub websocket: WebsocketConfig,
}

impl gravity::config::FromEnv for FromEnv {}

#[derive(
  Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct FromFile {
  /// UI config
  pub ui: UiConfig,
}

impl gravity::config::FromFile for FromFile {}

#[derive(Clone, Debug)]
pub struct Config {
  pub websocket: WebsocketConfig,
  pub ui: UiConfig,
}

impl gravity::config::Values for Config {
  type TArgs = FromArgs;
  type TEnv = FromEnv;
  type TFile = FromFile;

  fn new(_: Self::TArgs, env: Self::TEnv) -> Self {
    Self {
      websocket: env.websocket,
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

#[derive(derivative::Derivative, Clone, Debug, serde::Deserialize)]
#[derivative(Default)]
pub struct WebsocketConfig {
  #[derivative(Default(value = "\"localhost\".to_string()"))]
  pub host: String,
  #[derivative(Default(value = "5000"))]
  pub port: u16,
  #[derivative(Default(value = "true"))]
  pub ssl: bool,
}

#[derive(
  Default,
  Clone,
  Debug,
  serde::Serialize,
  serde::Deserialize,
  schemars::JsonSchema,
)]
pub struct UiConfig {
  /// UI colors
  pub palette: UiPaletteConfig,
}

#[derive(
  Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct UiPaletteConfig {
  /// Dark mode UI colors
  pub dark: UiPaletteModeConfig,
  /// Light mode UI colors
  pub light: UiPaletteModeConfig,
}

impl Default for UiPaletteConfig {
  fn default() -> Self {
    let dark = iced::Theme::Dark.palette();
    let light = iced::Theme::Light.palette();
    Self {
      dark: iced_palette_to_palette(&dark),
      light: iced_palette_to_palette(&light),
    }
  }
}

#[derive(
  Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct UiPaletteModeConfig {
  /// Background color
  #[schemars(schema_with = "palette_srgba_schema")]
  pub background: palette::rgb::Srgba,
  /// Text color
  #[schemars(schema_with = "palette_srgba_schema")]
  pub text: palette::rgb::Srgba,
  /// Primary color
  #[schemars(schema_with = "palette_srgba_schema")]
  pub primary: palette::rgb::Srgba,
  /// Success color
  #[schemars(schema_with = "palette_srgba_schema")]
  pub success: palette::rgb::Srgba,
  /// Failure color
  #[schemars(schema_with = "palette_srgba_schema")]
  pub danger: palette::rgb::Srgba,
}

fn palette_srgba_schema(
  gen: &mut schemars::gen::SchemaGenerator,
) -> schemars::schema::Schema {
  <String as schemars::JsonSchema>::json_schema(gen)
}

fn iced_palette_to_palette(
  palette: &iced::theme::Palette,
) -> UiPaletteModeConfig {
  UiPaletteModeConfig {
    background: iced_color_to_palette(&palette.background),
    text: iced_color_to_palette(&palette.text),
    primary: iced_color_to_palette(&palette.primary),
    success: iced_color_to_palette(&palette.success),
    danger: iced_color_to_palette(&palette.danger),
  }
}

fn iced_color_to_palette(color: &iced::Color) -> palette::Srgba {
  palette::Srgba::new(color.r, color.g, color.b, color.a)
}
