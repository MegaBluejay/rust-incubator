#![allow(dead_code)]

use std::{
    io::{Cursor, ErrorKind},
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
use image::{
    codecs::{
        jpeg::JpegDecoder,
        png::{self, PngDecoder, PngEncoder},
    },
    DynamicImage, ImageEncoder, ImageOutputFormat,
};
use isahc::{prelude::Configurable, HttpClient};
use once_cell::sync::Lazy;
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::{info, instrument, trace_span, Instrument};
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};

use config::{Cli, Config, SourceEnum};
use input_image::{ClientResult, InputImage};

mod config;
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
        jpeg_quality,
        png_compression,
        out_dir,
        max_concurrent,
        max_download_speed,
    } = config;

    let max_concurrent = max_concurrent.unwrap_or_else(num_cpus::get);
    let png_compression = png_compression.into();

    fs::create_dir_all(&out_dir).await?;

    let client = make_client(max_download_speed);
    let client_getter = || client.as_ref().map_err(Clone::clone);

    let mut results = into_input_images(source)
        .await?
        .err_into()
        .map_ok(|in_image| {
            process_image(
                in_image,
                &out_dir,
                jpeg_quality,
                png_compression,
                &client_getter,
            )
        })
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

    let mut figment = Figment::new().merge(Serialized::defaults(Config::default()));
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

enum ImageFormat {
    Jpeg,
    Png,
}

impl ImageFormat {
    fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
        }
    }
}

#[instrument(skip(client_getter), fields(out_dir = ?out_dir.as_ref()), err)]
async fn process_image<'a, F: FnOnce() -> ClientResult<'a>>(
    in_image: InputImage,
    out_dir: impl AsRef<Path>,
    jpeg_quality: u8,
    png_compression: png::CompressionType,
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

    let (out_data, format) =
        tokio_rayon::spawn(move || process_data(&in_data, jpeg_quality, png_compression)).await?;

    let mut out_file = create_out_file(out_dir.as_ref(), name, format.extension()).await?;

    out_file
        .write_all(&out_data)
        .instrument(trace_span!("write"))
        .await?;

    Ok(())
}

#[instrument]
async fn create_out_file(
    out_dir: &Path,
    name: &str,
    extension: &str,
) -> Result<File, std::io::Error> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    for out_path in out_paths(out_dir, name, extension) {
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

struct OutNames<'a> {
    name: &'a str,
    extension: &'a str,
    i: u32,
}

impl<'a> Iterator for OutNames<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let res = if self.i == 0 {
            format!("{}.{}", self.name, self.extension)
        } else {
            format!("{}({}).{}", self.name, self.i, self.extension)
        };
        self.i += 1;
        Some(res)
    }
}

impl<'a> OutNames<'a> {
    fn new(name: &'a str, extension: &'a str) -> Self {
        Self {
            name,
            extension,
            i: 0,
        }
    }
}

fn out_paths<'a>(
    out_dir: &'a Path,
    name: &'a str,
    extension: &'a str,
) -> impl Iterator<Item = PathBuf> + 'a {
    OutNames::new(name, extension).map(|out_name| out_dir.join(out_name))
}

#[instrument(skip(in_data))]
fn process_data(
    in_data: &[u8],
    quality: u8,
    png_compression: png::CompressionType,
) -> Result<(Vec<u8>, ImageFormat)> {
    let detect_span = trace_span!("detect").entered();
    let format = match imghdr::from_bytes(in_data) {
        Some(imghdr::Type::Png) => Ok(ImageFormat::Png),
        Some(imghdr::Type::Jpeg) => Ok(ImageFormat::Jpeg),
        Some(other) => Err(anyhow!("unsupported format: {:?}", other)),
        None => Err(anyhow!("unknown format")),
    }?;
    detect_span.exit();

    let decode_span = trace_span!("decode").entered();
    let image = match &format {
        ImageFormat::Jpeg => DynamicImage::from_decoder(JpegDecoder::new(in_data)?),
        ImageFormat::Png => DynamicImage::from_decoder(PngDecoder::new(in_data)?),
    }?;
    decode_span.exit();

    let _encode_span = trace_span!("encode").entered();
    let mut buffer = Cursor::new(vec![]);
    match &format {
        ImageFormat::Jpeg => image.write_to(&mut buffer, ImageOutputFormat::Jpeg(quality)),
        ImageFormat::Png => {
            let encoder =
                PngEncoder::new_with_quality(&mut buffer, png_compression, Default::default());
            encoder.write_image(
                image.as_bytes(),
                image.width(),
                image.height(),
                image.color(),
            )
        }
    }?;
    Ok((buffer.into_inner(), format))
}
