use clap::Parser;

mod cli;
mod config;

fn main() {
    let cli = cli::Cli::parse();
    let conf = config::TheConfig::new(&cli.options.conf);
    println!("{conf:#?}");
}
