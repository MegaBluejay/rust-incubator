#![allow(dead_code)]

use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use auto_enums::auto_enum;
use clap::Parser;
use figment::{
    providers::{Env, Format, Serialized, Yaml},
    Figment,
};
use futures::{stream::iter, Stream, StreamExt, TryStreamExt};
use futures_enum::Stream;
use isahc::{prelude::Configurable, HttpClient};
use once_cell::sync::Lazy;
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::{info, instrument, trace_span, Instrument};
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};

use cli::{Cli, Config, SourceEnum};
use input_image::{ClientResult, InputImage};

mod cli;
mod input_image;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE))
        .with(EnvFilter::from_default_env())
        .init();

    let (config, source) = get_config()?;

    info!(?config);

    let Config {
        quality,
        out_dir,
        max_concurrent,
        max_download_speed,
    } = config;

    fs::create_dir_all(&out_dir).await?;

    let client = make_client(max_download_speed);
    let client_getter = || client.as_ref().map_err(Clone::clone);

    let mut results = into_input_images(source)
        .await?
        .err_into()
        .map_ok(|in_image| process_image(in_image, &out_dir, quality, &client_getter))
        .try_buffer_unordered(max_concurrent);

    while results.next().await.is_some() {}

    Ok(())
}

fn get_config() -> Result<(Config, SourceEnum)> {
    let Cli {
        config: cli_config,
        config_file,
        source,
    } = Cli::try_parse()?;

    let mut figment = Figment::new();
    if let Some(config_file) = &config_file {
        figment = figment.merge(Yaml::file(config_file));
    }

    let config: Config = figment
        .merge(Env::prefixed("STEP3_"))
        .merge(Serialized::defaults(cli_config))
        .extract()?;
    Ok((config, source.into()))
}

fn make_client(
    max_download_speed: Option<u64>,
) -> Lazy<Result<HttpClient, isahc::Error>, impl Fn() -> Result<HttpClient, isahc::Error>> {
    Lazy::new(move || {
        let mut builder = HttpClient::builder();
        if let Some(max_download_speed) = max_download_speed {
            builder = builder.max_download_speed(max_download_speed);
        }
        builder.build()
    })
}

#[auto_enum(Stream)]
async fn into_input_images(
    source: SourceEnum,
) -> Result<impl Stream<Item = Result<InputImage, std::io::Error>>, std::io::Error> {
    Ok(
        #[nested]
        match source {
            SourceEnum::InputFile(file) => file.async_lines().await?.map_ok(Into::into),
            SourceEnum::Images(images) => iter(images.into_iter().map(Ok)),
        },
    )
}

#[instrument(skip(client_getter), fields(out_dir = ?out_dir.as_ref()), err)]
async fn process_image<'a, F: FnOnce() -> ClientResult<'a>>(
    in_image: InputImage,
    out_dir: impl AsRef<Path>,
    quality: f32,
    client_getter: F,
) -> Result<()> {
    let name = in_image
        .file_name()
        .and_then(|file_name| file_name.split('.').next())
        .unwrap_or("out");

    let mut in_data = vec![];
    in_image
        .open(client_getter)
        .instrument(trace_span!("open"))
        .await?
        .read_to_end(&mut in_data)
        .instrument(trace_span!("read"))
        .await?;

    let out_data = tokio_rayon::spawn(move || process_data(&in_data, quality)).await?;

    let mut out_file = create_out_file(out_dir.as_ref(), name).await?;

    out_file
        .write_all(&out_data)
        .instrument(trace_span!("write"))
        .await?;

    Ok(())
}

#[instrument]
async fn create_out_file(out_dir: &Path, name: &str) -> Result<File, std::io::Error> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    for out_path in out_paths(out_dir, name) {
        match options.open(&out_path).await {
            Ok(file) => {
                info!(?out_path);
                return Ok(file);
            }
            Err(err) => match err.kind() {
                ErrorKind::AlreadyExists => {}
                _ => return Err(err),
            },
        }
    }
    unreachable!()
}

struct OutNames {
    name: String,
    i: u32,
}

impl Iterator for OutNames {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let res = if self.i == 0 {
            format!("{}.jpg", self.name)
        } else {
            format!("{}({}).jpg", self.name, self.i)
        };
        self.i += 1;
        Some(res)
    }
}

impl OutNames {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            i: 0,
        }
    }
}

fn out_paths<'a>(
    out_dir: &'a Path,
    name: impl Into<String> + 'a,
) -> impl Iterator<Item = PathBuf> + 'a {
    OutNames::new(name).map(|out_name| out_dir.join(out_name))
}

#[instrument(skip(in_data, quality))]
fn process_data(in_data: &[u8], quality: f32) -> Result<Vec<u8>> {
    std::panic::catch_unwind(move || {
        let decompress_span = trace_span!("decompress").entered();
        let dec = mozjpeg::Decompress::with_markers(mozjpeg::ALL_MARKERS).from_mem(in_data)?;
        let mut image = dec.rgb()?;

        let width = image.width();
        let height = image.height();
        let color_space = image.color_space();
        info!(width, height, ?color_space);

        let pixels = image.read_scanlines_flat().unwrap();
        assert!(image.finish_decompress());
        decompress_span.exit();

        let _compress_span = trace_span!("compress").entered();
        let mut comp = mozjpeg::Compress::new(color_space);
        comp.set_size(width, height);
        comp.set_quality(quality);
        comp.set_mem_dest();
        comp.start_compress();

        assert!(comp.write_scanlines(&pixels));
        comp.finish_compress();

        Ok(comp.data_to_vec().unwrap())
    })
    .map_err(|_| anyhow!("mozjpeg error"))?
}
