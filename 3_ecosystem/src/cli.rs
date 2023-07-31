use std::path::PathBuf;

use clap::{Args, Parser};
use patharg::InputArg;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::input_image::InputImage;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(flatten)]
    pub config: OptConfig,
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

impl Source {
    pub fn into_enum(self) -> SourceEnum {
        if let Some(input_file) = self.input_file {
            SourceEnum::InputFile(input_file)
        } else {
            SourceEnum::Images(self.image.unwrap())
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Args, Serialize, Deserialize)]
pub struct OptConfig {
    #[arg(short, long)]
    pub quality: Option<f32>,
    #[arg(short, long)]
    pub out_dir: Option<PathBuf>,
    #[arg(short = 'j', long)]
    pub max_concurrent: Option<usize>,
    #[arg(short = 'd', long)]
    pub max_download_speed: Option<u64>,
}

#[derive(Debug)]
pub struct Config {
    pub quality: f32,
    pub out_dir: PathBuf,
    pub max_concurrent: usize,
    pub max_download_speed: Option<u64>,
}

#[derive(Debug)]
pub enum Missing {
    Quality,
    OutDir,
    MaxConcurrent,
}

impl OptConfig {
    pub fn unopt(self) -> Result<Config, Missing> {
        Ok(Config {
            quality: self.quality.ok_or(Missing::Quality)?,
            out_dir: self.out_dir.ok_or(Missing::OutDir)?,
            max_concurrent: self.max_concurrent.ok_or(Missing::MaxConcurrent)?,
            max_download_speed: self.max_download_speed,
        })
    }
}
