use std::{env, process};

use anyhow::Result;
use reqwest::StatusCode;

fn main() -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("http://0.0.0.0:3000")
        .json(&env::args().collect::<Vec<_>>())
        .send()?;

    match response.status() {
        StatusCode::OK => print!("{}", response.text()?),
        StatusCode::NO_CONTENT => {}
        _ => {
            eprint!("{}", response.text()?);
            process::exit(1);
        }
    }

    Ok(())
}
