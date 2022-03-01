use serde::Deserialize;
use web3::types::Address;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Networks
    pub networks: Vec<NetworkVaults>,

    /// Prometheus metrics exporter settings.
    pub metrics_settings: pomfrit::Config,

    /// log4rs settings.
    /// See [docs](https://docs.rs/log4rs/1.0.0/log4rs/) for more details
    #[serde(default = "default_logger_settings")]
    pub logger_settings: serde_yaml::Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkVaults {
    /// RPC endpoint
    pub endpoint: String,

    /// Vault addresses
    pub vaults: Vec<VaultsEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultsEntry {
    /// Vault address
    pub address: Address,

    /// Token group
    #[serde(default)]
    pub group: Option<String>,
}

fn default_logger_settings() -> serde_yaml::Value {
    const DEFAULT_LOG4RS_SETTINGS: &str = r##"
    appenders:
      stdout:
        kind: console
        encoder:
          pattern: "{d(%Y-%m-%d %H:%M:%S %Z)(utc)} - {h({l})} {M} = {m} {n}"
    root:
      level: error
      appenders:
        - stdout
    loggers:
      bridge_vaults_exporter:
        level: info
        appenders:
          - stdout
        additive: false
    "##;
    serde_yaml::from_str(DEFAULT_LOG4RS_SETTINGS).unwrap()
}
