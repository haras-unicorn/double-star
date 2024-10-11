pub trait FromArgs: clap::Args {}

pub trait FromEnv: serde::de::DeserializeOwned + Default {}

pub trait FromFile:
  serde::Serialize
  + serde::de::DeserializeOwned
  + schemars::JsonSchema
  + Send
  + Default
{
}

pub trait Values: Clone + Send {
  type TArgs: FromArgs;
  type TEnv: FromEnv;
  type TFile: FromFile;

  fn new(args: Self::TArgs, env: Self::TEnv) -> Self;

  fn import(&mut self, file: Self::TFile);

  fn export(&self) -> Self::TFile;
}

pub async fn new_async<T: Values + 'static>(
  prefix: &str,
  qualifier: &str,
  organization: &str,
  application: &str,
  root: &str,
) -> Config<T> {
  Config::new_async(prefix, qualifier, organization, application, root).await
}

pub fn new<T: Values + 'static>(
  prefix: &str,
  qualifier: &str,
  organization: &str,
  application: &str,
  root: &str,
) -> Config<T> {
  Config::new(prefix, qualifier, organization, application, root)
}

pub struct ConfigUpdate<T: Values + 'static> {
  pub config: T,
  pub error: Option<std::sync::Arc<anyhow::Error>>,
}

pub struct Config<T: Values + 'static> {
  values: std::sync::Arc<tokio::sync::Mutex<Wrapper<T>>>,
  #[allow(dead_code, reason = "just need to keep a handle somewhere")]
  watcher: Option<notify::INotifyWatcher>,
  #[allow(dead_code, reason = "just need to keep a handle somewhere")]
  tx: flume::Sender<ConfigUpdate<T>>,
  rx: flume::Receiver<ConfigUpdate<T>>,
  #[allow(dead_code, reason = "just need to keep a handle somewhere")]
  prefix: String,
  qualifier: String,
  organization: String,
  application: String,
}

impl<T: Values> Config<T> {
  pub fn values(&self) -> T {
    let lock = self.values.blocking_lock();
    lock.values.clone()
  }

  pub async fn values_async(&self) -> T {
    let lock = self.values.lock().await;
    lock.values.clone()
  }

  pub fn subscribe(&self) -> flume::Receiver<ConfigUpdate<T>> {
    self.rx.clone()
  }

  pub async fn subscribe_async(&self) -> flume::Receiver<ConfigUpdate<T>> {
    self.rx.clone()
  }

  pub fn import(&self) -> ConfigUpdate<T> {
    let mut lock = self.values.blocking_lock();
    match Self::reload_values(
      &mut *lock,
      &self.qualifier,
      &self.organization,
      &self.application,
    ) {
      Ok(_) => ConfigUpdate {
        config: lock.values.clone(),
        error: None,
      },
      Err(err) => ConfigUpdate {
        config: lock.values.clone(),
        error: Some(std::sync::Arc::new(err)),
      },
    }
  }

  pub async fn import_async(&self) -> ConfigUpdate<T> {
    let mut lock = self.values.lock().await;
    match Self::reload_values_async(
      &mut *lock,
      &self.qualifier,
      &self.organization,
      &self.application,
    )
    .await
    {
      Ok(_) => ConfigUpdate {
        config: lock.values.clone(),
        error: None,
      },
      Err(err) => ConfigUpdate {
        config: lock.values.clone(),
        error: Some(std::sync::Arc::new(err)),
      },
    }
  }

  pub fn export(&self, values: T) -> anyhow::Result<()> {
    let file = values.export();
    let mut lock = self.values.blocking_lock();
    let config_file = match Self::find_config_file(
      lock.config_path.clone(),
      &self.qualifier,
      &self.organization,
      &self.application,
    )
    .or_else(|| {
      Self::find_config_dir(
        &self.qualifier,
        &self.organization,
        &self.application,
      )
      .map(|config_dir| config_dir.join("config.toml"))
    }) {
      Some(config_file) => config_file,
      None => {
        return Ok(());
      }
    };

    match config_file.extension().and_then(|ext| ext.to_str()) {
      Some("yaml" | "yml") => {
        let yaml = serde_yaml::to_string(&file)?;
        std::fs::write(config_file, yaml)?;
      }
      Some("json") => {
        let json = serde_json::to_string(&file)?;
        std::fs::write(config_file, json)?;
      }
      Some("toml") => {
        let toml = toml::to_string(&file)?;
        std::fs::write(config_file, toml)?;
      }
      _ => {
        return Ok(());
      }
    };

    lock.values.import(file);

    Ok(())
  }

  pub async fn export_async(&self, values: T) -> anyhow::Result<()> {
    let file = values.export();
    let mut lock = self.values.blocking_lock();
    let config_file = match Self::find_config_file(
      lock.config_path.clone(),
      &self.qualifier,
      &self.organization,
      &self.application,
    )
    .or_else(|| {
      Self::find_config_dir(
        &self.qualifier,
        &self.organization,
        &self.application,
      )
      .map(|config_dir| config_dir.join("config.toml"))
    }) {
      Some(config_file) => config_file,
      None => {
        return Ok(());
      }
    };

    match config_file.extension().and_then(|ext| ext.to_str()) {
      Some("yaml" | "yml") => {
        let yaml = serde_yaml::to_string(&file)?;
        std::fs::write(config_file, yaml)?;
      }
      Some("json") => {
        let json = serde_json::to_string(&file)?;
        std::fs::write(config_file, json)?;
      }
      Some("toml") => {
        let toml = toml::to_string(&file)?;
        std::fs::write(config_file, toml)?;
      }
      _ => {
        return Ok(());
      }
    };

    lock.values.import(file);

    Ok(())
  }

  pub fn schema(&self) -> String {
    #[allow(
      clippy::unwrap_used,
      reason = "if this fails it is schemars' fault"
    )]
    serde_json::to_string_pretty(&schemars::schema_for!(FileWrapper<T::TFile>))
      .unwrap()
  }

  pub async fn schema_async(&self) -> String {
    #[allow(
      clippy::unwrap_used,
      reason = "if this fails it is schemars' fault"
    )]
    serde_json::to_string_pretty(&schemars::schema_for!(FileWrapper<T::TFile>))
      .unwrap()
  }

  fn new(
    prefix: &str,
    qualifier: &str,
    organization: &str,
    application: &str,
    root: &str,
  ) -> Self {
    let prefix = prefix.to_string();
    let qualifier = qualifier.to_string();
    let organization = organization.to_string();
    let application = application.to_string();

    let log_handle = match super::log::init(&prefix) {
      Ok(handle) => Some(handle),
      Err(err) => {
        tracing::error!("Logging already set up: {}", err);
        None
      }
    };

    let _ = dotenvy::dotenv();

    let raw_values =
      Self::load(&prefix, &qualifier, &organization, &application);

    if raw_values.print_schema {
      Self::print_schema()
    }
    Self::print_config(&raw_values, root);

    let watch_paths = Self::find_watch_paths(
      &raw_values,
      &qualifier,
      &organization,
      &application,
    );

    if let (Some(log_handle), Some(log_level)) =
      (&log_handle, &raw_values.log_level)
    {
      if let Err(err) =
        super::log::reload(&prefix, log_handle, log_level.into())
      {
        tracing::error!("Failed setting new log level {}", err);
      }
    }

    let values = std::sync::Arc::new(tokio::sync::Mutex::new(raw_values));
    let (tx, rx) = flume::unbounded();

    let watcher = Self::new_config_watcher(
      WatcherTask {
        values: values.clone(),
        qualifier: qualifier.clone(),
        organization: organization.clone(),
        application: application.to_string(),
        log_handle: log_handle.clone(),
        prefix: prefix.clone(),
        tx: tx.clone(),
      },
      watch_paths,
    );

    Self {
      values,
      watcher,
      tx,
      rx,
      prefix,
      qualifier,
      organization,
      application,
    }
  }

  async fn new_async(
    prefix: &str,
    qualifier: &str,
    organization: &str,
    application: &str,
    root: &str,
  ) -> Self {
    let prefix = prefix.to_string();
    let qualifier = qualifier.to_string();
    let organization = organization.to_string();
    let application = application.to_string();

    let log_handle = match super::log::init(&prefix) {
      Ok(handle) => Some(handle),
      Err(err) => {
        tracing::error!("Logging already set up: {}", err);
        None
      }
    };

    let raw_values =
      Self::load_async(&prefix, &qualifier, &organization, &application).await;

    if raw_values.print_schema {
      Self::print_schema()
    }
    Self::print_config(&raw_values, root);

    let watch_paths = Self::find_watch_paths_async(
      &raw_values,
      &qualifier,
      &organization,
      &application,
    )
    .await;

    if let (Some(log_handle), Some(log_level)) =
      (&log_handle, &raw_values.log_level)
    {
      if let Err(err) =
        super::log::reload(&prefix, log_handle, log_level.into())
      {
        tracing::error!("Failed setting new log level {}", err);
      }
    }

    let values = std::sync::Arc::new(tokio::sync::Mutex::new(raw_values));
    let (tx, rx) = flume::unbounded();

    let watcher = Self::new_config_watcher(
      WatcherTask {
        values: values.clone(),
        qualifier: qualifier.clone(),
        organization: organization.clone(),
        application: application.to_string(),
        log_handle: log_handle.clone(),
        prefix: prefix.clone(),
        tx: tx.clone(),
      },
      watch_paths,
    );

    Self {
      values,
      watcher,
      tx,
      rx,
      prefix,
      qualifier,
      organization,
      application,
    }
  }

  fn load(
    prefix: &str,
    qualifier: &str,
    organization: &str,
    application: &str,
  ) -> Wrapper<T> {
    let args = Self::parse_args();
    let env = Self::parse_env(prefix).unwrap_or_else(|err| {
      tracing::debug!("Using default env config: {}", err);
      Default::default()
    });
    let mut values = T::new(args.values, env.values);

    let mut file = None;
    if let Some(path) = Self::find_config_file(
      args.config_path.clone(),
      qualifier,
      organization,
      application,
    ) {
      if let Some(extension) = path.clone().extension().and_then(|x| x.to_str())
      {
        if let Ok(raw) = std::fs::read_to_string(path) {
          if let Ok(parsed) = Self::parse_file(raw.as_str(), extension) {
            file = Some(parsed);
          }
        }
      }
    }

    if let Some(file) = file {
      values.import(file.values);
      Wrapper {
        values,
        config_path: args.config_path.and(env.config_path),
        log_level: args.log_level.and(env.log_level).and(file.log_level),
        print_schema: args.print_schema,
        print_config: args.print_config,
      }
    } else {
      Wrapper {
        values,
        config_path: args.config_path.and(env.config_path),
        log_level: args.log_level.and(env.log_level),
        print_schema: args.print_schema,
        print_config: args.print_config,
      }
    }
  }

  async fn load_async(
    prefix: &str,
    qualifier: &str,
    organization: &str,
    application: &str,
  ) -> Wrapper<T> {
    let args = Self::parse_args_async().await;
    let env = Self::parse_env_async(prefix).await.unwrap_or_else(|err| {
      tracing::debug!("Using default env config: {}", err);
      Default::default()
    });
    let mut values = T::new(args.values, env.values);

    let mut file = None;
    if let Some(path) = Self::find_config_file_async(
      args.config_path.clone(),
      qualifier,
      organization,
      application,
    )
    .await
    {
      if let Some(extension) = path.clone().extension().and_then(|x| x.to_str())
      {
        if let Ok(raw) = tokio::fs::read_to_string(path).await {
          if let Ok(parsed) =
            Self::parse_file_async(raw.as_str(), extension).await
          {
            file = Some(parsed);
          }
        }
      }
    }

    if let Some(file) = file {
      values.import(file.values);
      Wrapper {
        values,
        config_path: args.config_path.and(env.config_path),
        log_level: args.log_level.and(env.log_level).and(file.log_level),
        print_schema: args.print_schema,
        print_config: args.print_config,
      }
    } else {
      Wrapper {
        values,
        config_path: args.config_path.and(env.config_path),
        log_level: args.log_level.and(env.log_level),
        print_schema: args.print_schema,
        print_config: args.print_config,
      }
    }
  }

  fn print_schema() {
    #[allow(
      clippy::unwrap_used,
      reason = "if this fails it is schemars' fault"
    )]
    let schema = serde_json::to_string_pretty(&schemars::schema_for!(
      FileWrapper<T::TFile>
    ))
    .unwrap();
    #[allow(clippy::print_stdout, reason = "this is exactly what we want")]
    {
      print!("{schema}");
    }
    std::process::exit(0);
  }

  fn print_config(values: &Wrapper<T>, root: &str) {
    let config_to_print = match values.print_config.as_deref() {
      Some("yaml" | "yml") => {
        let file = values.values.export();
        serde_yaml::to_string(&file)
          .map_err(|err| anyhow::format_err!(err))
          .map(|yaml| {
            Some(format!(
              "# yaml-language-server: $schema={root}/schema.json\n{yaml}"
            ))
          })
      }
      Some("json") => {
        let file = values.values.export();
        serde_json::to_value(&file)
          .map_err(|err| anyhow::format_err!(err))
          .map(|mut json| {
            let schema = format!("{root}/schema.json");
            #[allow(
              clippy::unwrap_used,
              reason = "Static string is valid json string"
            )]
            let json_value = serde_json::to_value(schema).unwrap();
            json["$schema"] = json_value;
            #[allow(clippy::unwrap_used, reason = "it is literally json")]
            let json_string = serde_json::to_string_pretty(&json).unwrap();
            Some(json_string)
          })
      }
      Some("toml") => {
        let file = values.values.export();
        toml::to_string(&file)
          .map_err(|err| anyhow::format_err!(err))
          .map(|toml| Some(format!("#:schema {root}/schema.json\n{toml}")))
      }
      Some(_) => Err(anyhow::anyhow!("Invalid config format")),
      None => Ok(None),
    };

    match config_to_print {
      Ok(None) => {}
      Ok(Some(config)) => {
        #[allow(clippy::print_stdout, reason = "this is exactly what we want")]
        {
          print!("{config}");
        }
        std::process::exit(0);
      }
      Err(err) => {
        #[allow(clippy::print_stderr, reason = "this is exactly what we want")]
        {
          eprint!("Config error: {err}");
        }
        std::process::exit(1);
      }
    }
  }

  fn reload_values(
    values: &mut Wrapper<T>,
    qualifier: &str,
    organization: &str,
    application: &str,
  ) -> anyhow::Result<()> {
    let path = if let Some(path) = Self::find_config_file(
      values.config_path.clone(),
      qualifier,
      organization,
      application,
    ) {
      path
    } else {
      return Ok(());
    };

    if let Some(extension) = path.clone().extension().and_then(|x| x.to_str()) {
      if let Ok(raw) = std::fs::read_to_string(path) {
        if let Ok(file) = Self::parse_file(raw.as_str(), extension) {
          values.values.import(file.values);
        }
      }
    }

    Ok(())
  }

  async fn reload_values_async(
    values: &mut Wrapper<T>,
    qualifier: &str,
    organization: &str,
    application: &str,
  ) -> anyhow::Result<()> {
    let path = if let Some(path) = Self::find_config_file_async(
      values.config_path.clone(),
      qualifier,
      organization,
      application,
    )
    .await
    {
      path
    } else {
      return Ok(());
    };

    if let Some(extension) = path.clone().extension().and_then(|x| x.to_str()) {
      if let Ok(raw) = tokio::fs::read_to_string(path).await {
        if let Ok(file) = Self::parse_file(raw.as_str(), extension) {
          values.values.import(file.values);
        }
      }
    }

    Ok(())
  }

  fn parse_args() -> ArgWrapper<T::TArgs> {
    clap::Parser::parse()
  }

  async fn parse_args_async() -> ArgWrapper<T::TArgs> {
    clap::Parser::parse()
  }

  fn parse_env(prefix: &str) -> anyhow::Result<EnvWrapper<T::TEnv>> {
    envy::prefixed(prefix)
      .from_env()
      .map_err(|err| anyhow::format_err!(err))
  }

  async fn parse_env_async(
    prefix: &str,
  ) -> anyhow::Result<EnvWrapper<T::TEnv>> {
    envy::prefixed(prefix)
      .from_env()
      .map_err(|err| anyhow::format_err!(err))
  }

  fn parse_file(
    raw: &str,
    extension: &str,
  ) -> anyhow::Result<FileWrapper<T::TFile>> {
    let values = match extension {
      "toml" => toml::from_str::<FileWrapper<T::TFile>>(raw)?,
      "yaml" | "yml" => serde_yaml::from_str::<FileWrapper<T::TFile>>(raw)?,
      "json" => serde_json::from_str::<FileWrapper<T::TFile>>(raw)?,
      _ => return Err(anyhow::anyhow!("Invalid config file extension")),
    };

    Ok(values)
  }

  async fn parse_file_async(
    raw: &str,
    extension: &str,
  ) -> anyhow::Result<FileWrapper<T::TFile>> {
    let values = match extension {
      "toml" => toml::from_str::<FileWrapper<T::TFile>>(raw)?,
      "yaml" | "yml" => serde_yaml::from_str::<FileWrapper<T::TFile>>(raw)?,
      "json" => serde_json::from_str::<FileWrapper<T::TFile>>(raw)?,
      _ => return Err(anyhow::anyhow!("Invalid config file extension")),
    };

    Ok(values)
  }

  fn new_config_watcher(
    task: WatcherTask<T>,
    paths: Vec<std::path::PathBuf>,
  ) -> Option<notify::INotifyWatcher> {
    let watcher = match notify::recommended_watcher(
      move |res: Result<notify::Event, notify::Error>| match res {
        Ok(event) => {
          if matches!(
            event.kind,
            notify::EventKind::Create(notify::event::CreateKind::File)
              | notify::EventKind::Modify(
                notify::event::ModifyKind::Data(
                  notify::event::DataChange::Content
                ) | notify::event::ModifyKind::Name(_)
              )
          ) {
            let mut lock = task.values.blocking_lock();
            let reload_err = if let Err(err) = Self::reload_values(
              &mut *lock,
              &task.qualifier,
              &task.organization,
              &task.application,
            ) {
              Some(std::sync::Arc::new(err))
            } else {
              None
            };

            if let (Some(log_handle), Some(log_level)) =
              (&task.log_handle, &lock.log_level)
            {
              if let Err(err) =
                super::log::reload(&task.prefix, log_handle, log_level.into())
              {
                tracing::error!("Failed setting new log level {}", err);
              }
            }

            if let Err(err) = task.tx.send(ConfigUpdate {
              config: lock.values.clone(),
              error: reload_err,
            }) {
              tracing::error!("Config watcher error: {}", err);
            }
          }
        }
        Err(err) => {
          tracing::error!("Config watcher error: {}", err);
        }
      },
    ) {
      Ok(mut watcher) => {
        for watch_path in paths {
          if let Err(err) = notify::Watcher::watch(
            &mut watcher,
            &watch_path,
            notify::RecursiveMode::NonRecursive,
          ) {
            tracing::error!("Config watcher error: {}", err);
          }
        }
        Some(watcher)
      }
      Err(err) => {
        tracing::error!("Config watcher error: {}", err);
        None
      }
    };
    watcher
  }

  fn find_watch_paths(
    values: &Wrapper<T>,
    qualifier: &str,
    organization: &str,
    application: &str,
  ) -> Vec<std::path::PathBuf> {
    if let Some(location) = values.config_path.clone() {
      let path = std::path::PathBuf::from(location);
      return match path.parent() {
        Some(config_dir) => vec![config_dir.to_path_buf()],
        None => Vec::new(),
      };
    }

    let config_dir = if let Some(config_dir) =
      Self::find_config_dir(qualifier, organization, application)
    {
      config_dir
    } else {
      return Vec::new();
    };

    if matches!(std::fs::exists(&config_dir), Ok(false)) {
      if let Err(err) = std::fs::create_dir(&config_dir) {
        tracing::error!("Failed creating config dir: {}", err);
      }
    }

    vec![config_dir]
  }

  async fn find_watch_paths_async(
    values: &Wrapper<T>,
    qualifier: &str,
    organization: &str,
    application: &str,
  ) -> Vec<std::path::PathBuf> {
    if let Some(location) = values.config_path.clone() {
      let path = std::path::PathBuf::from(location);
      return match path.parent() {
        Some(config_dir) => vec![config_dir.to_path_buf()],
        None => Vec::new(),
      };
    }

    let config_dir = if let Some(config_dir) =
      Self::find_config_dir(qualifier, organization, application)
    {
      config_dir
    } else {
      return Vec::new();
    };

    if matches!(tokio::fs::try_exists(&config_dir).await, Ok(false)) {
      if let Err(err) = tokio::fs::create_dir(&config_dir).await {
        tracing::error!("Failed creating config dir: {}", err);
      }
    }

    vec![config_dir]
  }

  fn find_config_file(
    location: Option<String>,
    qualifier: &str,
    organization: &str,
    application: &str,
  ) -> Option<std::path::PathBuf> {
    if let Some(location) = location {
      if std::fs::exists(location.clone()).is_ok_and(|x| x) {
        return Some(std::path::PathBuf::from(location));
      }
    }

    let config_dir = if let Some(config_dir) =
      Self::find_config_dir(qualifier, organization, application)
    {
      if std::fs::exists(&config_dir).is_ok_and(|x| x) {
        config_dir
      } else {
        return None;
      }
    } else {
      return None;
    };

    Self::find_config_file_in_config_dir(config_dir)
  }

  async fn find_config_file_async(
    location: Option<String>,
    qualifier: &str,
    organization: &str,
    application: &str,
  ) -> Option<std::path::PathBuf> {
    if let Some(location) = location {
      if tokio::fs::try_exists(location.clone())
        .await
        .is_ok_and(|x| x)
      {
        return Some(std::path::PathBuf::from(location));
      }
    }

    let config_dir = if let Some(config_dir) =
      Self::find_config_dir(qualifier, organization, application)
    {
      if tokio::fs::try_exists(&config_dir).await.is_ok_and(|x| x) {
        config_dir
      } else {
        return None;
      }
    } else {
      return None;
    };

    Self::find_config_file_in_config_dir_async(config_dir).await
  }

  fn find_config_file_in_config_dir(
    config_dir: std::path::PathBuf,
  ) -> Option<std::path::PathBuf> {
    let possible_config_files =
      if let Ok(possible_config_files) = std::fs::read_dir(config_dir) {
        possible_config_files
      } else {
        return None;
      };

    for possible_config_file in possible_config_files.into_iter() {
      match possible_config_file {
        Ok(possible_config_file) => {
          let possible_config_file: std::path::PathBuf =
            possible_config_file.file_name().into();
          if Self::is_config_file(possible_config_file.clone()) {
            return Some(possible_config_file);
          }
        }
        Err(_) => {
          break;
        }
      };
    }

    None
  }

  async fn find_config_file_in_config_dir_async(
    config_dir: std::path::PathBuf,
  ) -> Option<std::path::PathBuf> {
    let mut possible_config_files = if let Ok(possible_config_files) =
      tokio::fs::read_dir(config_dir).await
    {
      possible_config_files
    } else {
      return None;
    };

    while let Ok(Some(possible_config_file)) =
      possible_config_files.next_entry().await
    {
      let possible_config_file: std::path::PathBuf =
        possible_config_file.file_name().into();
      if Self::is_config_file(possible_config_file.clone()) {
        return Some(possible_config_file);
      }
    }

    None
  }

  fn is_config_file(path: std::path::PathBuf) -> bool {
    let stem =
      if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
        stem
      } else {
        return false;
      };

    let extension =
      if let Some(stem) = path.extension().and_then(|stem| stem.to_str()) {
        stem
      } else {
        return false;
      };

    matches!(
      (stem, extension),
      ("config", "yaml" | "yml" | "json" | "toml")
    )
  }

  fn find_config_dir(
    qualifier: &str,
    organization: &str,
    application: &str,
  ) -> Option<std::path::PathBuf> {
    let project_dirs =
      directories::ProjectDirs::from(qualifier, organization, application)?;

    let config_dir = project_dirs.config_dir().to_path_buf();

    Some(config_dir)
  }
}

#[derive(clap::Parser)]
struct ArgWrapper<T: FromArgs + 'static> {
  #[clap(flatten)]
  values: T,
  /// Config file path
  #[clap(long)]
  config_path: Option<String>,
  /// Change log level
  #[clap(long, value_parser = clap::builder::PossibleValuesParser::new(["trace", "debug", "info", "warn", "error"]))]
  log_level: Option<String>,
  /// Print config file schema
  #[clap(long, action)]
  print_schema: bool,
  /// Print current file config
  #[clap(long, value_parser = clap::builder::PossibleValuesParser::new(["yaml", "yml", "toml", "json"]))]
  print_config: Option<String>,
}

#[derive(serde::Deserialize, Default)]
#[serde(bound = "T: serde::de::DeserializeOwned")]
struct EnvWrapper<T: FromEnv + 'static> {
  #[serde(flatten)]
  values: T,
  /// Config file path
  config_path: Option<String>,
  /// Change log level
  log_level: Option<LogLevel>,
}

#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(bound = "T: serde::Serialize + serde::de::DeserializeOwned")]
#[doc = concat!("# ", env!("CARGO_PKG_NAME"))]
#[doc = env!("CARGO_PKG_DESCRIPTION")]
struct FileWrapper<T: FromFile + 'static> {
  #[serde(flatten)]
  values: T,
  /// Change log level
  log_level: Option<LogLevel>,
}

struct Wrapper<T: Values + 'static> {
  values: T,
  config_path: Option<String>,
  log_level: Option<LogLevel>,
  print_schema: bool,
  print_config: Option<String>,
}

struct WatcherTask<T: Values + 'static> {
  values: std::sync::Arc<tokio::sync::Mutex<Wrapper<T>>>,
  qualifier: String,
  organization: String,
  application: String,
  log_handle: Option<
    tracing_subscriber::reload::Handle<
      tracing_subscriber::EnvFilter,
      tracing_subscriber::Registry,
    >,
  >,
  prefix: String,
  tx: flume::Sender<ConfigUpdate<T>>,
}

#[derive(
  serde::Serialize,
  serde::Deserialize,
  strum::EnumString,
  strum::IntoStaticStr,
  schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
enum LogLevel {
  #[strum(serialize = "trace")]
  Trace,
  #[strum(serialize = "debug")]
  Debug,
  #[strum(serialize = "info")]
  Info,
  #[strum(serialize = "warn")]
  Warn,
  #[strum(serialize = "error")]
  Error,
}
