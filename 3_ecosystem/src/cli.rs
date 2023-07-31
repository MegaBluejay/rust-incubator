use std::path::PathBuf;

use clap::{Args, Parser};
use patharg::InputArg;

use super::input_image::InputImage;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub quality: f32,
    #[arg(short, long)]
    pub out_dir: PathBuf,
    #[arg(short = 'j', long)]
    pub max_concurrent: usize,
    #[command(flatten)]
    pub source: Source,
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
