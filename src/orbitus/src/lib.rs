#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::todo)]
#![deny(clippy::unreachable)]
#![deny(clippy::allow_attributes_without_reason)]

mod app;
pub mod config;
pub mod ws;

pub fn run(
  double_star_tx: flume::Sender<gravity::OrbitusMessage>,
  double_star_rx: flume::Receiver<gravity::DoubleStarMessage>,
  config: config::Config,
  config_tx: flume::Sender<config::Config>,
  config_rx: flume::Receiver<gravity::config::ConfigUpdate<config::Config>>,
) -> anyhow::Result<()> {
  Ok(
    iced::application::application(
      app::Orbitus::title,
      app::Orbitus::update,
      app::Orbitus::view,
    )
    .subscription(app::Orbitus::subscription)
    .theme(app::Orbitus::theme)
    .run_with(move || {
      app::Orbitus::new(
        double_star_tx,
        double_star_rx,
        config,
        config_tx,
        config_rx,
      )
    })?,
  )
}
