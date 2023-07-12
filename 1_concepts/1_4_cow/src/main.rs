use std::{
    borrow::Cow,
    env,
    path::{Path, PathBuf},
};

use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    conf: Option<PathBuf>,
}

fn config_path(arg_conf: Option<PathBuf>) -> Cow<'static, Path> {
    if let Some(arg_conf) = arg_conf {
        return Cow::Owned(arg_conf);
    }

    if let Ok(env_conf) = env::var("APP_CONF") {
        return Cow::Owned(env_conf.into());
    }

    Cow::Borrowed(Path::new("/etc/app/app.conf"))
}

fn main() {
    let args = Args::parse();
    println!("{:?}", config_path(args.conf));
}
