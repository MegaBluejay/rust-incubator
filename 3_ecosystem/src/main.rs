use std::{io::Cursor, path::Path};

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
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::{debug_span, info, instrument, Instrument};
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};

use config::{Cli, Config, SourceEnum};
use input_image::{ClientResult, InputImage};
use out_file::create_out_file;

mod config;
mod input_image;
mod out_file;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE))
        .with(EnvFilter::from_default_env())
        .init();

    let (config, source) = Cli::try_parse().map_err(Into::into).and_then(get_config)?;

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

fn get_config(cli: Cli) -> Result<(Config, SourceEnum)> {
    let Cli {
        config: cli_config,
        config_file,
        source,
    } = cli;

    // I went with `figment` instead of `config` for configuration merging
    // because its `Serialized` source is convenient for values defined in code
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
        .instrument(debug_span!("open"))
        .await?
        .read_to_end(&mut in_data)
        .instrument(debug_span!("read"))
        .await?;

    // since decoding/encoding an image is a cpu-bound task
    // using a thread pool like `rayon` seems like a good idea
    let (out_data, format) =
        tokio_rayon::spawn(move || process_data(&in_data, jpeg_quality, png_compression)).await?;

    let mut out_file = create_out_file(out_dir.as_ref(), name, format.extension()).await?;

    out_file
        .write_all(&out_data)
        .instrument(debug_span!("write"))
        .await?;

    info!("processed");

    Ok(())
}

#[instrument(skip(in_data))]
fn process_data(
    in_data: &[u8],
    quality: u8,
    png_compression: png::CompressionType,
) -> Result<(Vec<u8>, ImageFormat)> {
    // determining the file type from header bytes instead of extension is more reliable,
    // and allows this to work even when we can't extract a filename from a url
    let format = debug_span!("detect").in_scope(|| match imghdr::from_bytes(in_data) {
        Some(imghdr::Type::Png) => Ok(ImageFormat::Png),
        Some(imghdr::Type::Jpeg) => Ok(ImageFormat::Jpeg),
        Some(other) => Err(anyhow!("unsupported format: {:?}", other)),
        None => Err(anyhow!("unknown format")),
    })?;

    let image = debug_span!("decode").in_scope(|| match &format {
        ImageFormat::Jpeg => DynamicImage::from_decoder(JpegDecoder::new(in_data)?),
        ImageFormat::Png => DynamicImage::from_decoder(PngDecoder::new(in_data)?),
    })?;

    let out_data = debug_span!("encode").in_scope(|| {
        let mut buffer = Cursor::new(vec![]);
        match &format {
            ImageFormat::Jpeg => image.write_to(&mut buffer, ImageOutputFormat::Jpeg(quality)),
            ImageFormat::Png => {
                // can't use the `write_to` method for pngs because it doesn't allow compression configuration
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
        Ok::<Vec<u8>, anyhow::Error>(buffer.into_inner())
    })?;

    Ok((out_data, format))
}

#[cfg(test)]
mod tests {
    use crate::config::{CompressionType, OptionalConfig, Source};

    use super::*;

    #[test]
    fn config_priority() {
        figment::Jail::expect_with(|jail| {
            let config_file = "config.yaml".into();

            jail.create_file(
                &config_file,
                "jpeg_quality: 10\npng_compression: fast\nmax_concurrent: 3",
            )?;
            jail.set_env("STEP3_JPEG_QUALITY", 20);
            jail.set_env("STEP3_PNG_COMPRESSION", "best");

            let cli = Cli {
                config: OptionalConfig {
                    jpeg_quality: Some(30),
                    ..Default::default()
                },
                source: Source {
                    input_file: Some(patharg::InputArg::Path("list".into())),
                    image: None,
                },
                config_file: Some(config_file),
            };

            let (config, _source) = get_config(cli)
                .map_err(|_| figment::Error::from("config parsing error".to_owned()))?;

            assert_eq!(config.jpeg_quality, 30);
            assert_eq!(config.png_compression, CompressionType::Best);
            assert_eq!(config.max_concurrent, Some(3));

            Ok(())
        });
    }
}
