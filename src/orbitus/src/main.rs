#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::todo)]
#![deny(clippy::unreachable)]
#![deny(clippy::allow_attributes_without_reason)]

fn main() -> anyhow::Result<()> {
  let config = gravity::config::new::<orbitus::config::Config>(
    "ORBITUS",
    env!("QUALIFIER"),
    env!("ORGANIZATION"),
    "orbitus",
    concat!(env!("CARGO_PKG_REPOSITORY"), "/src/orbitus"),
  );
  let config_values = config.values();
  let config_rx = config.subscribe();

  let (double_star_tx, double_star_rx) =
    flume::unbounded::<gravity::DoubleStarMessage>();
  let (orbitus_tx, orbitus_rx) = flume::unbounded::<gravity::OrbitusMessage>();
  let (config_tx, config_subscriber) = flume::unbounded();

  let ws_config = config.values();
  let ws_handle = std::thread::spawn(move || {
    if let Err(err) = orbitus::ws::run(double_star_tx, orbitus_rx, ws_config) {
      tracing::error!("Websocket failed: {err}");
    }
  });

  let config_handle = std::thread::spawn(move || {
    while let Ok(new_config) = config_subscriber.recv() {
      if let Err(err) = config.export(new_config) {
        tracing::error!("Config error: {}", err);
      }
    }
  });

  {
    let orbitus_tx = orbitus_tx.clone();
    orbitus::run(
      orbitus_tx,
      double_star_rx,
      config_values,
      config_tx,
      config_rx,
    )?;
  }

  orbitus_tx.send(gravity::OrbitusMessage::Exited)?;

  if let Err(err) = ws_handle.join() {
    return Err(anyhow::anyhow!("Join failed: {err:?}"));
  }

  if let Err(err) = config_handle.join() {
    return Err(anyhow::anyhow!("Join failed: {err:?}"));
  }

  Ok(())
}
