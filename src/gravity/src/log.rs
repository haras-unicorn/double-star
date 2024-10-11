use tracing_subscriber::{
  layer::SubscriberExt, reload::Handle, util::SubscriberInitExt, EnvFilter,
  Registry,
};

pub fn init(prefix: &str) -> anyhow::Result<Handle<EnvFilter, Registry>> {
  #[cfg(not(debug_assertions))]
  let level = "info";
  #[cfg(debug_assertions)]
  let level = "debug";

  let format_layer = tracing_subscriber::fmt::layer();
  let (filter_layer, filter_handle) = tracing_subscriber::reload::Layer::new(
    build_tracing_filter(prefix, &level)?,
  );
  tracing_subscriber::registry()
    .with(filter_layer)
    .with(format_layer)
    .try_init()?;

  Ok(filter_handle)
}

pub fn reload(
  prefix: &str,
  handle: &Handle<EnvFilter, Registry>,
  level: &str,
) -> anyhow::Result<()> {
  let log_level = level.to_string();
  handle.modify(move |filter| {
    #[allow(clippy::unwrap_used, reason = "static and env doesn't change")]
    let new_filter = build_tracing_filter(prefix, log_level.as_str()).unwrap();
    *filter = new_filter;
  })?;

  Ok(())
}

fn build_tracing_filter(
  prefix: &str,
  level: &str,
) -> anyhow::Result<EnvFilter> {
  Ok(
    tracing_subscriber::EnvFilter::builder()
      .with_default_directive(tracing::level_filters::LevelFilter::WARN.into())
      .with_env_var(format!("{prefix}_LOG"))
      .from_env()?
      .add_directive(format!("gravity={level}").parse()?)
      .add_directive(format!("orbitus={level}").parse()?)
      .add_directive(format!("double_star={level}").parse()?)
      .add_directive(format!("nebulon={level}").parse()?),
  )
}
