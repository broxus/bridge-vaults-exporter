use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use argh::FromArgs;
use serde::Deserialize;

use self::config::*;
use self::service::*;

mod config;
mod contracts;
mod printed_num;
mod service;

#[tokio::main]
async fn main() -> Result<()> {
    let app = argh::from_env::<App>();
    run(app).await
}

async fn run(app: App) -> Result<()> {
    let config: Config = read_config(app.config)?;
    init_logger(&config.logger_settings)?;

    let service = Service::new(config.networks)
        .await
        .context("Failed to create service")?;

    let interval = Duration::from_secs(config.metrics_settings.collection_interval_sec);
    service.start_listening(interval).await?;

    log::info!(
        "Server is running on {} with interval {}s",
        config.metrics_settings.listen_address,
        interval.as_secs()
    );

    let (_exporter, writer) = pomfrit::create_exporter(Some(config.metrics_settings)).await?;

    writer.spawn(move |buffer| {
        buffer.write(service.metrics());
    });

    futures::future::pending().await
}

#[derive(Debug, PartialEq, FromArgs)]
#[argh(description = "Octus Bridge vaults info exporter")]
struct App {
    /// path to the application config
    #[argh(option, short = 'c', default = "PathBuf::from(\"config.yaml\")")]
    config: PathBuf,
}

fn read_config<P, T>(path: P) -> Result<T>
where
    P: AsRef<std::path::Path>,
    for<'de> T: Deserialize<'de>,
{
    let data = std::fs::read_to_string(path).context("Failed to read config")?;
    let re = regex::Regex::new(r"\$\{([a-zA-Z_][0-9a-zA-Z_]*)\}").unwrap();
    let result = re.replace_all(&data, |caps: &regex::Captures| {
        match std::env::var(&caps[1]) {
            Ok(value) => value,
            Err(_) => {
                eprintln!("WARN: Environment variable {} was not set", &caps[1]);
                String::default()
            }
        }
    });

    let mut config = ::config::Config::new();
    config.merge(::config::File::from_str(
        result.as_ref(),
        ::config::FileFormat::Yaml,
    ))?;

    config.try_into().context("Failed to parse config")
}

fn init_logger(initial_value: &serde_yaml::Value) -> Result<log4rs::Handle> {
    let handle = log4rs::config::init_config(parse_logger_config(initial_value.clone())?)?;
    Ok(handle)
}

fn parse_logger_config(value: serde_yaml::Value) -> Result<log4rs::Config> {
    let config = serde_yaml::from_value::<log4rs::config::RawConfig>(value)?;

    let (appenders, errors) = config.appenders_lossy(&log4rs::config::Deserializers::default());
    if !errors.is_empty() {
        return Err(InitError::Deserializing).with_context(|| format!("{:#?}", errors));
    }

    let config = log4rs::Config::builder()
        .appenders(appenders)
        .loggers(config.loggers())
        .build(config.root())?;
    Ok(config)
}

#[derive(thiserror::Error, Debug)]
enum InitError {
    #[error("Errors found when deserializing the logger config")]
    Deserializing,
}
