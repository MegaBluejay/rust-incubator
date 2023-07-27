use std::{net::Ipv4Addr, path::Path, time::Duration};

use config::{Config, ConfigError};
use serde::Deserialize;
use smart_default::SmartDefault;
use url::Url;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct TheConfig {
    pub mode: Mode,
    pub server: Server,
    pub db: Db,
    pub log: Logs,
    pub background: Background,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Mode {
    pub debug: bool,
}

#[derive(Debug, Deserialize, SmartDefault)]
#[serde(default)]
pub struct Server {
    #[default("http://127.0.0.1".try_into().unwrap())]
    pub external_url: Url,
    #[default(8081)]
    pub http_port: u16,
    #[default(8082)]
    pub grpc_port: u16,
    #[default(10025)]
    pub healthz_port: u16,
    #[default(9199)]
    pub metrics_port: u16,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Db {
    pub mysql: MySql,
}

#[derive(Debug, Deserialize, SmartDefault)]
#[serde(default)]
pub struct MySql {
    #[default(url::Host::Ipv4(Ipv4Addr::new(127, 0, 0, 1)))]
    pub host: url::Host,
    #[default(3306)]
    pub port: u16,
    #[default("default")]
    pub dating: String,
    #[default("root")]
    pub user: String,
    pub pass: String,
}

#[derive(Debug, Deserialize, SmartDefault)]
#[serde(default)]
pub struct MySqlConnections {
    #[default(30)]
    pub max_idle: usize,
    #[default(30)]
    pub max_open: usize,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Logs {
    pub app: Log,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Log {
    pub level: LogLevel,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Background {
    pub watchdog: Watchdog,
}

#[derive(Debug, Deserialize, SmartDefault)]
#[serde(default)]
pub struct Watchdog {
    #[serde(with = "humantime_serde")]
    #[default(Duration::from_secs(5))]
    pub period: Duration,
    #[default(10)]
    pub limit: usize,
    #[serde(with = "humantime_serde")]
    #[default(Duration::from_secs(4))]
    pub lock_timeout: Duration,
}

impl TheConfig {
    pub fn new(conf: &Path) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(config::File::from(conf).required(false))
            .add_source(config::Environment::with_prefix("CONF"))
            .build()?
            .try_deserialize()
    }
}
