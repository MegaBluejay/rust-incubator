use std::path::PathBuf;

use clap::{Args, Parser, ValueEnum};
use image::codecs::png;
use optional_struct::{optional_struct, Applyable};
use patharg::InputArg;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::input_image::InputImage;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(flatten)]
    pub config: OptionalConfig,
    #[command(flatten)]
    pub source: Source,
    #[arg(long)]
    pub config_file: Option<PathBuf>,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct Source {
    #[arg(short = 'f', long)]
    pub input_file: Option<InputArg>,
    pub image: Option<Vec<InputImage>>,
}

pub enum SourceEnum {
    InputFile(InputArg),
    Images(Vec<InputImage>),
}

impl From<Source> for SourceEnum {
    fn from(value: Source) -> Self {
        // clap currently doesn't have a way of using an enum for a `multiple = false` `ArgGroup`,
        // but since it guarantees only one of the fields is present, this is valid
        if let Some(input_file) = value.input_file {
            Self::InputFile(input_file)
        } else {
            Self::Images(value.image.unwrap())
        }
    }
}

#[optional_struct]
#[skip_serializing_none]
#[derive(Debug, Args, Serialize, Deserialize)]
pub struct Config {
    #[arg(short = 'q', long)]
    pub jpeg_quality: u8,
    #[arg(short = 'c', long)]
    pub png_compression: CompressionType,
    #[arg(short, long)]
    pub out_dir: PathBuf,
    #[arg(short = 'j', long)]
    pub max_concurrent: Option<usize>,
    #[arg(short = 'd', long)]
    pub max_download_speed: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            jpeg_quality: 80,
            png_compression: CompressionType::Default,
            out_dir: "./out".into(),
            max_concurrent: None,
            max_download_speed: None,
        }
    }
}

// A separate enum is neccessary to derive the instances
// I'm also not including deprecated types
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum CompressionType {
    Default,
    Fast,
    Best,
}

impl From<CompressionType> for png::CompressionType {
    fn from(value: CompressionType) -> Self {
        match value {
            CompressionType::Default => png::CompressionType::Default,
            CompressionType::Fast => png::CompressionType::Fast,
            CompressionType::Best => png::CompressionType::Best,
        }
    }
}
