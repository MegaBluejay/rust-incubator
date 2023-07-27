use std::{net::Ipv4Addr, path::Path, time::Duration};

use clap::Parser;
use config::{Config, ConfigError};
use serde::Deserialize;
use smart_default::SmartDefault;
use url::Url;

mod cli;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct TheConfig {
    mode: Mode,
    server: Server,
    db: Db,
    log: Logs,
    background: Background,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct Mode {
    debug: bool,
}

#[derive(Debug, Deserialize, SmartDefault)]
#[serde(default)]
struct Server {
    #[default("http://127.0.0.1".try_into().unwrap())]
    external_url: Url,
    #[default(8081)]
    http_port: u16,
    #[default(8082)]
    grpc_port: u16,
    #[default(10025)]
    healthz_port: u16,
    #[default(9199)]
    metrics_port: u16,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct Db {
    mysql: MySql,
}

#[derive(Debug, Deserialize, SmartDefault)]
#[serde(default)]
struct MySql {
    #[default(url::Host::Ipv4(Ipv4Addr::new(127, 0, 0, 1)))]
    host: url::Host,
    #[default(3306)]
    port: u16,
    #[default("default")]
    dating: String,
    #[default("root")]
    user: String,
    pass: String,
}

#[derive(Debug, Deserialize, SmartDefault)]
#[serde(default)]
struct MySqlConnections {
    #[default(30)]
    max_idle: usize,
    #[default(30)]
    max_open: usize,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct Logs {
    app: Log,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct Log {
    level: LogLevel,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct Background {
    watchdog: Watchdog,
}

#[derive(Debug, Deserialize, SmartDefault)]
#[serde(default)]
struct Watchdog {
    #[serde(with = "humantime_serde")]
    #[default(Duration::from_secs(5))]
    period: Duration,
    #[default(10)]
    limit: usize,
    #[serde(with = "humantime_serde")]
    #[default(Duration::from_secs(4))]
    lock_timeout: Duration,
}

impl TheConfig {
    fn new(conf: &Path) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(config::File::from(conf).required(false))
            .add_source(config::Environment::with_prefix("CONF"))
            .build()?
            .try_deserialize()
    }
}

fn main() {
    let cli = cli::Cli::parse();
    let conf = TheConfig::new(&cli.options.conf);
    println!("{conf:#?}");
}
