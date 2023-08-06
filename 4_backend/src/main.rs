use std::env;

use anyhow::Result;
use sea_orm::Database;

use step_4::server::server;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let db_url = env::var("DATABASE_URL")?;
    let db = Database::connect(&db_url).await?;

    server(db, &"0.0.0.0:3000".parse()?, "secret".as_bytes()).await?;

    Ok(())
}
