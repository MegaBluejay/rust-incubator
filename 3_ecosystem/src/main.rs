#![allow(dead_code)]

use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use auto_enums::auto_enum;
use bytes::BytesMut;
use clap::Parser;
use cli::{Config, OptConfig, SourceEnum};
use figment::{
    providers::{Env, Format, Serialized, Yaml},
    Figment,
};
use futures::{stream::iter, Stream, StreamExt, TryStreamExt};
use futures_enum::Stream;
use input_image::InputImage;
use tokio::{
    fs::{self, File, OpenOptions},
    io::AsyncWriteExt,
};

mod cli;
mod global_client;
mod input_image;

#[tokio::main]
async fn main() -> Result<()> {
    let cli::Cli {
        config,
        config_file,
        source,
    } = cli::Cli::parse();

    let opt_config: OptConfig = tokio::task::spawn_blocking(move || {
        let mut figment = Figment::new();
        if let Some(config_file) = &config_file {
            figment = figment.merge(Yaml::file(config_file));
        }
        figment
            .merge(Env::prefixed("STEP3_"))
            .merge(Serialized::defaults(config))
            .extract()
    })
    .await??;

    let Config {
        quality,
        out_dir,
        max_concurrent,
    } = opt_config
        .unopt()
        .map_err(|missing| anyhow!("missing config value: {:?}", missing))?;

    fs::create_dir_all(&out_dir).await?;

    let mut results = into_input_images(source.into_enum())
        .await?
        .err_into()
        .map_ok(|in_image| process_image(in_image, &out_dir, quality))
        .try_buffer_unordered(max_concurrent);

    while let Some(result) = results.next().await {
        if let Err(err) = result {
            eprintln!("{err}");
        }
    }

    Ok(())
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

async fn process_image(
    in_image: InputImage,
    out_dir: impl AsRef<Path>,
    quality: f32,
) -> Result<()> {
    let name = in_image
        .file_name()
        .and_then(|file_name| file_name.split('.').next())
        .unwrap_or("out")
        .to_owned();

    let in_data: BytesMut = in_image.bytes_stream().await?.try_collect().await?;

    let out_data = tokio_rayon::spawn(move || process_data(&in_data, quality)).await?;

    let mut out_file = create_out_file(out_dir.as_ref(), &name).await?;

    out_file.write_all(&out_data).await?;

    Ok(())
}

async fn create_out_file(out_dir: &Path, name: &str) -> Result<File, std::io::Error> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    for out_path in out_paths(out_dir, name) {
        match options.open(&out_path).await {
            Ok(file) => return Ok(file),
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

fn process_data(in_data: &[u8], quality: f32) -> Result<Vec<u8>> {
    std::panic::catch_unwind(move || {
        let dec = mozjpeg::Decompress::with_markers(mozjpeg::ALL_MARKERS).from_mem(in_data)?;
        let mut image = dec.rgb()?;

        let width = image.width();
        let height = image.height();
        let color_space = image.color_space();

        let pixels = image.read_scanlines_flat().unwrap();
        assert!(image.finish_decompress());

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
