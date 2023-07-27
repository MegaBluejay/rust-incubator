use std::borrow::Cow;
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
enum Request<'a> {
    Success {
        #[serde(borrow)]
        stream: Stream<'a>,
        #[serde(borrow)]
        gifts: Vec<Gift<'a>>,
        debug: DebugInfo,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Stream<'a> {
    user_id: Uuid,
    is_private: bool,
    settings: u32,
    shard_url: Url,
    #[serde(borrow)]
    public_tariff: PublicTariff<'a>,
    #[serde(borrow)]
    private_tariff: PrivateTariff<'a>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct PublicTariff<'a> {
    #[serde(flatten, borrow)]
    base: BaseTariff<'a>,
    id: u32,
    price: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct PrivateTariff<'a> {
    #[serde(flatten, borrow)]
    base: BaseTariff<'a>,
    client_price: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct BaseTariff<'a> {
    #[serde(with = "humantime_serde")]
    duration: Duration,
    #[serde(borrow)]
    description: Cow<'a, str>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Gift<'a> {
    id: u32,
    price: u32,
    #[serde(borrow)]
    description: Cow<'a, str>,
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

    macro_rules! check_borrowed {
        ($($cow:expr),+ $(,)?) => {
            $(
                if let Cow::Owned(_) = $cow {
                    panic!("{} is owned", stringify!($cow));
                }
            )+
        };
    }

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
                        description: "test public tariff".into(),
                    },
                    id: 1,
                    price: 100,
                },
                private_tariff: PrivateTariff {
                    base: BaseTariff {
                        duration: Duration::from_secs(60),
                        description: "test private tariff".into(),
                    },
                    client_price: 250,
                },
            },
            gifts: vec![
                Gift {
                    id: 1,
                    price: 2,
                    description: "Gift 1".into(),
                },
                Gift {
                    id: 2,
                    price: 3,
                    description: "Gift 2".into(),
                },
            ],
            debug: DebugInfo {
                duration: Duration::from_millis(234),
                at: datetime!(2019-06-28 08:35:46 UTC),
            },
        };

        assert_eq!(request, expected);

        let Request::Success {
            stream,
            gifts,
            debug: _,
        } = request;

        check_borrowed!(
            stream.public_tariff.base.description,
            stream.private_tariff.base.description,
            gifts[0].description,
            gifts[1].description,
        );

        Ok(())
    }
}
