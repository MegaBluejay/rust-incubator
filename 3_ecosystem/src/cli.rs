use std::path::PathBuf;

use clap::{Args, Parser};
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
    input_file: Option<InputArg>,
    image: Option<Vec<InputImage>>,
}

pub enum SourceEnum {
    InputFile(InputArg),
    Images(Vec<InputImage>),
}

impl From<Source> for SourceEnum {
    fn from(value: Source) -> Self {
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
    #[arg(short, long)]
    pub quality: f32,
    #[arg(short, long)]
    pub out_dir: PathBuf,
    #[arg(short = 'j', long)]
    pub max_concurrent: usize,
    #[arg(short = 'd', long)]
    pub max_download_speed: Option<u64>,
}
