#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::todo)]
#![deny(clippy::unreachable)]
#![deny(clippy::allow_attributes_without_reason)]

fn main() -> anyhow::Result<()> {
  let config = gravity::config::new::<double_star::config::Config>(
    "DOUBLE_STAR",
    env!("QUALIFIER"),
    env!("ORGANIZATION"),
    "double-star",
    concat!(env!("CARGO_PKG_REPOSITORY"), "/src/double-star"),
  );
  let config_values = config.values();
  let orbitus_config_values = config.values();
  let config_rx = config.subscribe();
  let orbitus_config_rx = config.subscribe();

  let (double_star_tx, double_star_rx) = flume::unbounded();
  let (orbitus_tx, orbitus_rx) = flume::unbounded();
  let (config_tx, config_subscriber) =
    flume::unbounded::<orbitus::config::Config>();
  let (orbitus_mapped_config_tx, orbitus_mapped_config_rx) = flume::unbounded();

  let double_star_handle = std::thread::spawn(move || {
    if let Err(err) =
      double_star::run(double_star_tx, orbitus_rx, config_values, config_rx)
    {
      tracing::error!("Double Starr error: {}", err);
    }
  });

  let config_handle = std::thread::spawn(move || {
    while let Ok(new_config) = config_subscriber.recv() {
      if let Err(err) = config.export(double_star::config::Config {
        db: config.values().db,
        ui: new_config.ui,
      }) {
        tracing::error!("Config error: {}", err);
      }
    }
  });

  let orbitus_mapped_config_handle = std::thread::spawn(move || {
    while let Ok(gravity::config::ConfigUpdate { config, error }) =
      orbitus_config_rx.recv()
    {
      {
        if let Err(err) =
          orbitus_mapped_config_tx.send(gravity::config::ConfigUpdate {
            config: orbitus::config::Config {
              websocket: Default::default(),
              ui: config.ui,
            },
            error,
          })
        {
          tracing::error!("Config error: {}", err);
        }
      }
    }
  });

  {
    let orbitus_tx = orbitus_tx.clone();
    orbitus::run(
      orbitus_tx,
      double_star_rx,
      orbitus::config::Config {
        websocket: Default::default(),
        ui: orbitus_config_values.ui,
      },
      config_tx,
      orbitus_mapped_config_rx,
    )?;
  }

  if let Err(flume::TrySendError::Full(_)) =
    orbitus_tx.try_send(gravity::OrbitusMessage::Exited)
  {
    return Err(anyhow::anyhow!("Failed sending orbitus exit message"));
  }

  if let Err(err) = double_star_handle.join() {
    return Err(anyhow::anyhow!("Failed joining double star {:?}", err));
  }

  if let Err(err) = config_handle.join() {
    return Err(anyhow::anyhow!("Failed joining config {:?}", err));
  }

  if let Err(err) = orbitus_mapped_config_handle.join() {
    return Err(anyhow::anyhow!("Failed joining mapped config {:?}", err));
  }

  Ok(())
}
