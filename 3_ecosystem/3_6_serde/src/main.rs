use std::time::Duration;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

static INPUT: &[u8] = include_bytes!("../request.json");

fn main() -> Result<(), anyhow::Error> {
    let req: Request = serde_json::from_slice(INPUT)?;

    serde_yaml::to_writer(std::io::stdout(), &req)?;

    println!("-----");

    let toml_out = toml::to_string_pretty(&req)?;
    println!("{toml_out}");

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Request {
    Success {
        stream: Stream,
        gifts: Vec<Gift>,
        debug: DebugInfo,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Stream {
    user_id: Uuid,
    is_private: bool,
    settings: u32,
    shard_url: Url,
    public_tariff: PublicTariff,
    private_tariff: PrivateTariff,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct PublicTariff {
    #[serde(flatten)]
    base: BaseTariff,
    id: u32,
    price: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct PrivateTariff {
    #[serde(flatten)]
    base: BaseTariff,
    client_price: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct BaseTariff {
    #[serde(with = "humantime_serde")]
    duration: Duration,
    description: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Gift {
    id: u32,
    price: u32,
    description: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DebugInfo {
    #[serde(with = "humantime_serde")]
    duration: Duration,
    #[serde(with = "time::serde::rfc3339")]
    at: OffsetDateTime,
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;
    use uuid::uuid;

    use super::*;

    #[test]
    fn request() -> Result<(), anyhow::Error> {
        let request: Request = serde_json::from_slice(INPUT)?;
        let expected = Request::Success {
            stream: Stream {
                user_id: uuid!("8d234120-0bda-49b2-b7e0-fbd3912f6cbf"),
                is_private: false,
                settings: 45345,
                shard_url: Url::parse("https://n3.example.com/sapi").unwrap(),
                public_tariff: PublicTariff {
                    base: BaseTariff {
                        duration: Duration::from_secs(60 * 60),
                        description: "test public tariff".to_owned(),
                    },
                    id: 1,
                    price: 100,
                },
                private_tariff: PrivateTariff {
                    base: BaseTariff {
                        duration: Duration::from_secs(60),
                        description: "test private tariff".to_owned(),
                    },
                    client_price: 250,
                },
            },
            gifts: vec![
                Gift {
                    id: 1,
                    price: 2,
                    description: "Gift 1".to_owned(),
                },
                Gift {
                    id: 2,
                    price: 3,
                    description: "Gift 2".to_owned(),
                },
            ],
            debug: DebugInfo {
                duration: Duration::from_millis(234),
                at: datetime!(2019-06-28 08:35:46 UTC),
            },
        };
        assert_eq!(request, expected);
        Ok(())
    }
}
