use std::time::Duration;

use anyhow::Error;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

fn main() -> Result<(), Error> {
    let input = include_bytes!("../request.json");

    let req: Request = serde_json::from_slice(input)?;

    serde_yaml::to_writer(std::io::stdout(), &req)?;

    println!("-----");

    let toml_out = toml::to_string_pretty(&req)?;
    println!("{toml_out}");

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    #[serde(rename = "type")]
    req_type: ReqType,
    stream: Stream,
    gifts: Vec<Gift>,
    debug: DebugInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReqType {
    Success,
}

#[derive(Debug, Serialize, Deserialize)]
struct Stream {
    user_id: Uuid,
    is_private: bool,
    settings: u32,
    shard_url: Url,
    public_tariff: PublicTariff,
    private_tariff: PrivateTariff,
}

#[derive(Debug, Serialize, Deserialize)]
struct PublicTariff {
    #[serde(flatten)]
    base: BaseTariff,
    id: u32,
    price: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct PrivateTariff {
    #[serde(flatten)]
    base: BaseTariff,
    client_price: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct BaseTariff {
    #[serde(with = "humantime_serde")]
    duration: Duration,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Gift {
    id: u32,
    price: u32,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DebugInfo {
    #[serde(with = "humantime_serde")]
    duration: Duration,
    #[serde(with = "time::serde::rfc3339")]
    at: OffsetDateTime,
}
