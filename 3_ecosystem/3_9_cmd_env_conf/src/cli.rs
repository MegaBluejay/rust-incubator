use std::path::PathBuf;

use clap::{
    builder::styling::{Style, Styles},
    ArgAction, Args, Parser,
};
use indoc::indoc;

#[derive(Debug, Args)]
pub struct Flags {
    /// Enables debug mode
    #[arg(short, long)]
    pub debug: bool,
    /// Prints help information
    #[arg(short, long, action = ArgAction::Help)]
    pub help: (),
    /// Prints version information
    #[arg(short = 'V', long, action = ArgAction::Version)]
    pub version: (),
}

#[derive(Debug, Args)]
pub struct Options {
    /// Path to configuration file
    #[arg(
        short,
        long,
        id = "conf",
        env = "CONF_FILE",
        default_value = "config.toml"
    )]
    pub conf: PathBuf,
}

/// Prints its configuration to STDOUT
#[derive(Debug, Parser)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    styles = Styles::default().header(Style::new()),
    version = "0.1.0",
    override_usage = "step_3_9 [FLAGS] [OPTIONS]",
    help_template = indoc! {"
    {about-section}
    USAGE:
        {usage}

    {all-args}
    "}
)]
pub struct Cli {
    #[command(flatten, next_help_heading = "FLAGS")]
    pub flags: Flags,
    #[command(flatten, next_help_heading = "OPTIONS")]
    pub options: Options,
}
