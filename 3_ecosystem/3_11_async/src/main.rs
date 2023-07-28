use std::path::{Path, PathBuf};

use clap::Parser;

use anyhow::{Context, Result};
use filenamify::filenamify;
use futures::TryStreamExt;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    runtime,
    task::JoinSet,
};
use tracing::instrument;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long)]
    max_threads: Option<usize>,
    #[arg(id = "FILE")]
    path: PathBuf,
}

fn main() -> Result<()> {
    let Cli { max_threads, path } = Cli::parse();

    let mut rt_builder = runtime::Builder::new_multi_thread();
    rt_builder.enable_all();
    if let Some(max_threads) = max_threads {
        rt_builder.worker_threads(max_threads);
    }

    let rt = rt_builder.build()?;

    rt.block_on(async_main(&path))
}

async fn async_main(path: &Path) -> Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new())?;

    let client = reqwest::Client::new();

    let file = File::open(path).await.context("couldn't open urls file")?;
    let mut lines = BufReader::new(file).lines();

    let mut join_set = JoinSet::new();

    while let Some(line) = lines
        .next_line()
        .await
        .context("failed to read from urls file")?
    {
        let task_client = client.clone();
        join_set.spawn(async move {
            let _ = handle(task_client, &line).await;
        });
    }

    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }

    Ok(())
}

#[instrument(skip(client), err)]
async fn handle(client: reqwest::Client, line: &str) -> Result<()> {
    let mut filename = filenamify(line);
    if !filename.ends_with(".html") {
        filename.push_str(".html");
    }

    let response = client.get(line).send().await?.error_for_status()?;

    let file = File::create(filename)
        .await
        .context("failed to create output file")?;
    let mut writer = BufWriter::new(file);

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.try_next().await.context("failed receiving data")? {
        writer
            .write_all(&chunk)
            .await
            .context("failed writing data")?;
    }
    writer.flush().await.context("failed writing data")?;
    Ok(())
}
