use std::path::{Path, PathBuf};

use clap::Parser;

use anyhow::Result;
use filenamify::filenamify;
use futures::TryStreamExt;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    runtime,
    task::JoinSet,
};

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
    let client = reqwest::Client::new();

    let file = File::open(path).await?;
    let mut lines = BufReader::new(file).lines();

    let mut join_set = JoinSet::new();

    while let Some(line) = lines.next_line().await? {
        let task_client = client.clone();
        join_set.spawn(async move {
            if let Err(err) = handle(task_client, line).await {
                eprintln!("{err}");
            }
        });
    }

    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }

    Ok(())
}

async fn handle(client: reqwest::Client, line: String) -> Result<()> {
    let filename = filenamify(&line);

    let response = client.get(line).send().await?.error_for_status()?;

    let file = File::create(filename).await?;
    let mut writer = BufWriter::new(file);

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.try_next().await? {
        writer.write_all(&chunk).await?;
    }
    writer.flush().await?;
    Ok(())
}
