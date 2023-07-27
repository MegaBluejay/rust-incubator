use std::path::PathBuf;

use clap::{
    builder::styling::{Style, Styles},
    ArgAction, Args, Parser,
};
use indoc::indoc;

#[derive(Debug, Args)]
struct Flags {
    /// Enables debug mode
    #[arg(short, long)]
    debug: bool,
    /// Prints help information
    #[arg(short, long, action = ArgAction::Help)]
    help: (),
    /// Prints version information
    #[arg(short = 'V', long, action = ArgAction::Version)]
    version: (),
}

#[derive(Debug, Args)]
struct Options {
    /// Path to configuration file
    #[arg(
        short,
        long,
        id = "conf",
        env = "CONF_FILE",
        default_value = "config.toml"
    )]
    conf: PathBuf,
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
struct Cli {
    #[command(flatten, next_help_heading = "FLAGS")]
    flags: Flags,
    #[command(flatten, next_help_heading = "OPTIONS")]
    options: Options,
}

fn main() {
    println!("{:?}", Cli::parse());
}
